using System.Reflection;
using System.Text.Json;
using Launcher.Core;

if (args.Length > 0 && args[0] == "--child")
{
    await Task.Delay(250);
    File.WriteAllText(args[1], JsonSerializer.Serialize(args.Skip(2).ToArray()));
    return;
}

var root = Path.Combine(Path.GetTempPath(), "launcher-checks-" + Guid.NewGuid().ToString("N"));
Directory.CreateDirectory(root);
var checks = 0;
void Check(bool condition, string message) { if (!condition) throw new Exception(message); checks++; }
void Expect<T>(Action action) where T : Exception
{
    try { action(); } catch (T) { checks++; return; }
    throw new Exception($"Expected {typeof(T).Name}");
}

try
{
    var navigation = new NavigationState(3);
    navigation.Apply(NavigationAction.Left, 0);
    Check(navigation.ConsoleIndex == 2, "Wrap backwards");
    navigation.Apply(NavigationAction.Right, 0);
    Check(navigation.ConsoleIndex == 0, "Wrap forwards");
    Check(!navigation.Apply(NavigationAction.Confirm, 2), "First confirmation enters games");
    navigation.Apply(NavigationAction.Down, 2);
    Check(navigation.GameIndex == 1 && navigation.Apply(NavigationAction.Confirm, 2), "Second confirmation requests selected game");
    navigation.Apply(NavigationAction.Back, 2);
    Check(navigation.View == LibraryView.Consoles && navigation.ConsoleIndex == 0, "Back preserves console");
    navigation.Apply(NavigationAction.Confirm, 0);
    Check(!navigation.Apply(NavigationAction.Confirm, 0), "Empty list cannot launch");

    var store = new LibraryStore(Path.Combine(root, "library.json"));
    var data = store.Load();
    var rom = Path.Combine(root, "A game & special ' characters.rvz");
    File.WriteAllText(rom, "fixture, not a real game");
    var imported = LibraryStore.ImportGameCube(data, rom);
    LibraryStore.ImportGameCube(data, rom);
    Check(data.Games.Count == 1, "Import is idempotent");
    data.DolphinPath = "configured";
    store.Save(data);
    Check(store.Load().Games.Single() == imported && store.Load().DolphinPath == "configured", "Roundtrip library and settings");
    var invalid = Path.Combine(root, "bad.txt"); File.WriteAllText(invalid, "fixture");
    Expect<InvalidDataException>(() => LibraryStore.ImportGameCube(data, invalid));
    File.WriteAllText(Path.Combine(root, "library.json"), "broken");
    Expect<JsonException>(() => store.Load());
    Check(File.ReadAllText(Path.Combine(root, "library.json")) == "broken", "Loading does not destroy invalid file");
    File.WriteAllText(Path.Combine(root, "library.json"), "{\"Games\":null}");
    Expect<InvalidDataException>(() => store.Load());
    File.WriteAllText(Path.Combine(root, "library.json"), "{\"Games\":[{}]}");
    Expect<InvalidDataException>(() => store.Load());

    var executable = Path.Combine(root, "Dolphin.exe"); File.WriteAllText(executable, "fixture");
    var adapter = new DolphinAdapter();
    var request = adapter.Prepare(imported, executable);
    Check(request.Arguments.Contains("--batch") && request.Arguments[2] == rom && request.Arguments.Contains("Dolphin.Display.Fullscreen=True"), "Batch, exact ROM argument and fullscreen");
    Expect<InvalidOperationException>(() => adapter.Prepare(imported with { SystemId = "ps2" }, executable));
    Expect<FileNotFoundException>(() => adapter.Prepare(imported with { FilePath = null }, executable));
    Expect<FileNotFoundException>(() => adapter.Prepare(imported, Path.Combine(root, "missing.exe")));

    var runner = new SessionRunner();
    try { await runner.RunAsync(new(Path.Combine(root, "missing.exe"), [], root)); }
    catch (System.ComponentModel.Win32Exception) { checks++; }
    Check(!runner.IsRunning, "Failed start releases session");
    var output = Path.Combine(root, "child output.json");
    var host = Environment.GetEnvironmentVariable("DOTNET_HOST_PATH") ?? "dotnet";
    var launch = new LaunchRequest(host, [Assembly.GetExecutingAssembly().Location, "--child", output, rom, "literal & argument"], root);
    var first = runner.RunAsync(launch);
    Check(runner.IsRunning, "Session marked running");
    try { await runner.RunAsync(launch); throw new Exception("Concurrent session allowed"); }
    catch (InvalidOperationException) { checks++; }
    Check(await first == 0 && !runner.IsRunning, "Process exit releases session");
    Check(JsonSerializer.Deserialize<string[]>(File.ReadAllText(output))!.SequenceEqual(new[] { rom, "literal & argument" }), "Arguments survive spaces and shell metacharacters");
    Console.WriteLine($"PASS: {checks} checks (navigation, storage, adapter, real child process).");
}
finally
{
    // Only this test's newly created, uniquely named temporary directory is removed.
    Directory.Delete(root, recursive: true);
}
