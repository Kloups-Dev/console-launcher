using System.Reflection;
using System.IO;
using System.Text.Json;
using System.Windows;
using System.Windows.Threading;
using Launcher.Core;

if (args.Length > 0 && args[0] == "--child")
{
    await Task.Delay(250);
    File.WriteAllText(args[1], JsonSerializer.Serialize(args.Skip(2).ToArray()));
    return;
}

if (args.Length > 0 && args[0] == "--window-child")
{
    // A real, offscreen window checks graceful WM_CLOSE handling without an emulator or ROM.
    var marker = args[1];
    var thread = new Thread(() =>
    {
        var application = new Application();
        var window = new Window { Title = "Launcher session check", Width = 120, Height = 80,
            Left = -20000, Top = -20000, ShowInTaskbar = true, WindowStartupLocation = WindowStartupLocation.Manual };
        var expired = false;
        var timeout = new DispatcherTimer { Interval = TimeSpan.FromSeconds(10) };
        window.Loaded += (_, _) => { File.WriteAllText(marker + ".ready", "ready"); timeout.Start(); };
        timeout.Tick += (_, _) => { expired = true; window.Close(); };
        window.Closed += (_, _) => { timeout.Stop(); File.WriteAllText(marker, expired ? "timeout" : "closed normally"); };
        application.Run(window);
    });
    thread.SetApartmentState(ApartmentState.STA);
    thread.Start();
    thread.Join();
    return;
}

var root = Path.Combine(Path.GetTempPath(), "launcher-checks-" + Guid.NewGuid().ToString("N"));
Directory.CreateDirectory(root);
var checks = 0;
Task<int>? windowSessionForCleanup = null;
void Check(bool condition, string message) { if (!condition) throw new Exception(message); checks++; }
void Expect<T>(Action action) where T : Exception
{
    try { action(); } catch (T) { checks++; return; }
    throw new Exception($"Expected {typeof(T).Name}");
}

try
{
    var hold = new HoldGesture(1500);
    Check(!hold.Update(true, 1000) && !hold.Update(true, 2499), "Short holds do not request exit");
    Check(hold.Update(true, 2500), "Continuous hold requests exit at threshold");
    Check(!hold.Update(true, 5000), "Held gesture fires once");
    hold.Update(false, 5010);
    Check(!hold.Update(true, 5020) && hold.Update(true, 6520), "Release rearms gesture");
    hold.Update(false, 7000);
    hold.Update(true, 7010);
    hold.Update(false, 8000);
    Check(!hold.Update(true, 9000) && !hold.Update(true, 10000), "Interrupted holds do not accumulate");
    hold.Reset();
    Check(!hold.Update(true, 11000) && !hold.Update(true, 12000), "Reset clears previous controller hold");

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
    Check(request.Arguments.Contains("--batch") && request.Arguments[2] == rom && request.Arguments.Contains("Dolphin.Display.Fullscreen=True")
        && request.Arguments.Contains("Dolphin.Interface.ConfirmStop=False"), "Batch, exact ROM argument, fullscreen and session-only stop confirmation");
    Expect<InvalidOperationException>(() => adapter.Prepare(imported with { SystemId = "ps2" }, executable));
    Expect<FileNotFoundException>(() => adapter.Prepare(imported with { FilePath = null }, executable));
    Expect<FileNotFoundException>(() => adapter.Prepare(imported, Path.Combine(root, "missing.exe")));

    var runner = new SessionRunner();
    Check(!runner.RequestStop(), "No session: exit request ignored");
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
    Check(!runner.RequestStop(), "Exited session: exit request ignored");

    var backendPath = Path.Combine(root, "backend", "library.json");
    var fakeRunner = new ControlledSession();
    var backend = new LauncherBackend(new LibraryStore(backendPath), fakeRunner);
    var observed = new List<SessionPhase>();
    backend.Changed += (_, _) => observed.Add(backend.Status.Phase);
    Check(backend.Import("gamecube", [rom, rom]) == 1, "Backend imports duplicate paths once");
    var backendGame = backend.Games("gamecube").Single();
    Check(!backend.Prepare(backendGame.Id).CanLaunch, "Backend reports missing configuration before launch");
    await backend.StartAsync(backendGame.Id);
    Check(backend.Status.Phase == SessionPhase.Failed && fakeRunner.Calls == 0, "Blocked preparation never executes a process");
    backend.Configure("gamecube", new("dolphin", executable));
    Check(backend.Prepare(backendGame.Id).CanLaunch, "Backend prepares configured game");
    var backendSession = backend.StartAsync(backendGame.Id);
    Check(backend.Status.Phase == SessionPhase.Running && fakeRunner.Calls == 1, "Backend publishes running state after starting");
    Expect<InvalidOperationException>(() => backend.Configure("gamecube", new("dolphin", executable)));
    Expect<InvalidOperationException>(() => backend.Remove(backendGame.Id));
    try { await backend.StartAsync(backendGame.Id); throw new Exception("Backend allowed concurrent session"); }
    catch (InvalidOperationException) { checks++; }
    Check(backend.RequestStop() && backend.Status.Phase == SessionPhase.StopRequested, "Backend requests graceful stop and waits");
    fakeRunner.Complete(0);
    await backendSession;
    Check(backend.Status.Phase == SessionPhase.Completed && observed.Contains(SessionPhase.Preparing), "Backend completes state lifecycle");
    fakeRunner.FailOnStart = true;
    await backend.StartAsync(backendGame.Id);
    Check(backend.Status.Phase == SessionPhase.Failed && !backend.Status.Busy, "Backend recovers from start failure");
    var reloaded = new LauncherBackend(new LibraryStore(backendPath));
    Check(reloaded.Configuration("gamecube")?.Executable == executable && reloaded.Games("gamecube").Count == 1, "Backend persists configuration and collection");
    reloaded.Remove(backendGame.Id);
    Check(File.Exists(rom) && reloaded.Games("gamecube").Count == 0, "Removing an entry preserves the game file");
    Expect<InvalidOperationException>(() => reloaded.Configure("ps5", new("dolphin", executable)));
    Check(reloaded.Import("gamecube", [rom]) == 1, "Backend can import again after removal");
    Expect<InvalidDataException>(() => reloaded.Import("gamecube", [rom, invalid]));
    Check(reloaded.Games("gamecube").Count == 1, "Failed batch import preserves in-memory collection");
    var corruptPath = Path.Combine(root, "corrupt.json");
    File.WriteAllText(corruptPath, "invalid");
    var corruptBackend = new LauncherBackend(new LibraryStore(corruptPath));
    Check(!corruptBackend.CanWrite && corruptBackend.Status.Phase == SessionPhase.Failed, "Corrupt library is read-only");
    Expect<InvalidOperationException>(() => corruptBackend.Import("gamecube", [rom]));
    Check(File.ReadAllText(corruptPath) == "invalid", "Corrupt original remains unchanged");
    var unwritablePath = Path.Combine(root, "unwritable-library.json");
    var unwritable = new LauncherBackend(new LibraryStore(unwritablePath));
    Directory.CreateDirectory(unwritablePath);
    Expect<UnauthorizedAccessException>(() => unwritable.Import("gamecube", [rom]));
    Check(unwritable.Games("gamecube").Count == 0, "Failed persistence never applies an import in memory");
    var legacyPath = Path.Combine(root, "legacy.json");
    File.WriteAllText(legacyPath, JsonSerializer.Serialize(new { DolphinPath = executable, Games = new[] { imported } }));
    var legacy = new LauncherBackend(new LibraryStore(legacyPath));
    Check(legacy.Prepare(imported.Id).CanLaunch, "Old Dolphin configuration remains usable");
    var ps2File = Path.Combine(root, "PS2 game.iso"); File.WriteAllText(ps2File, "fixture");
    var ps2Executable = Path.Combine(root, "pcsx2-qt.exe"); File.WriteAllText(ps2Executable, "fixture");
    var launchPlanner = new LaunchPlanner();
    var ps2Plan = launchPlanner.Prepare(new("ps2-fixture", "ps2", "PS2 fixture", ps2File), new("pcsx2", ps2Executable));
    Check(ps2Plan.Arguments[^2] == "--" && ps2Plan.Arguments[^1] == ps2File && ps2Plan.Arguments.Contains("-nogui"), "PCSX2 receives direct launch and separate file argument");
    var raExecutable = Path.Combine(root, "retroarch.exe"); File.WriteAllText(raExecutable, "fixture");
    var core = Path.Combine(root, "test_core.dll"); File.WriteAllText(core, "fixture");
    var snesFile = Path.Combine(root, "SNES game.sfc"); File.WriteAllText(snesFile, "fixture");
    var snesGame = new GameEntry("snes-fixture", "snes", "SNES fixture", snesFile);
    var raPlan = launchPlanner.Prepare(snesGame, new("retroarch", raExecutable, core));
    Check(raPlan.Arguments[1] == core && raPlan.Arguments[^1] == snesFile, "RetroArch receives core and game independently");
    Expect<FileNotFoundException>(() => launchPlanner.Prepare(snesGame, new("retroarch", raExecutable)));
    Expect<InvalidOperationException>(() => launchPlanner.Prepare(snesGame, new("dolphin", executable)));
    var unsupported = new GameEntry("ps5-fixture", "ps5", "Unavailable", ps2File);
    Expect<InvalidOperationException>(() => launchPlanner.Prepare(unsupported, new("pcsx2", ps2Executable)));

    var marker = Path.Combine(root, "graceful-close.txt");
    var windowLaunch = new LaunchRequest(host, [Assembly.GetExecutingAssembly().Location, "--window-child", marker], root);
    var windowSession = runner.RunAsync(windowLaunch);
    windowSessionForCleanup = windowSession;
    var readyUntil = DateTime.UtcNow.AddSeconds(8);
    while (!File.Exists(marker + ".ready") && !windowSession.IsCompleted && DateTime.UtcNow < readyUntil)
        await Task.Delay(50);
    Check(File.Exists(marker + ".ready"), "Real child window is ready");
    var stopSent = false;
    var stopUntil = DateTime.UtcNow.AddSeconds(2);
    while (!stopSent && !windowSession.IsCompleted && DateTime.UtcNow < stopUntil)
    { stopSent = runner.RequestStop(); if (!stopSent) await Task.Delay(50); }
    Check(stopSent, "Graceful window close request sent");
    Check(await windowSession.WaitAsync(TimeSpan.FromSeconds(5)) == 0, "Window session exits successfully");
    Check(File.ReadAllText(marker) == "closed normally" && !runner.IsRunning, "Close handler runs before launcher releases session");
    Console.WriteLine($"PASS: {checks} checks (navigation, storage, hold gesture, sessions and graceful window close).");
}
finally
{
    if (windowSessionForCleanup is not null && !windowSessionForCleanup.IsCompleted)
        await windowSessionForCleanup.WaitAsync(TimeSpan.FromSeconds(12));
    // Only this test's newly created, uniquely named temporary directory is removed.
    Directory.Delete(root, recursive: true);
}

internal sealed class ControlledSession : ISessionRunner
{
    private TaskCompletionSource<int>? completion;
    public int Calls { get; private set; }
    public bool FailOnStart { get; set; }
    public bool IsRunning => completion is not null && !completion.Task.IsCompleted;
    public Task<int> RunAsync(LaunchRequest request)
    {
        Calls++;
        if (FailOnStart) throw new System.ComponentModel.Win32Exception("Simulated start failure");
        completion = new(TaskCreationOptions.RunContinuationsAsynchronously);
        return completion.Task;
    }
    public bool RequestStop() => IsRunning;
    public void Complete(int exitCode) => completion!.SetResult(exitCode);
}
