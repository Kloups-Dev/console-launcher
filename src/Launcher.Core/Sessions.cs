using System.Diagnostics;

namespace Launcher.Core;

public sealed record LaunchRequest(string Executable, IReadOnlyList<string> Arguments, string WorkingDirectory);

public interface IEmulatorAdapter
{
    string Id { get; }
    LaunchRequest Prepare(GameEntry game, string executable);
}

public sealed class DolphinAdapter : IEmulatorAdapter
{
    public string Id => "dolphin";

    public LaunchRequest Prepare(GameEntry game, string executable)
        => Prepare(game, executable, null);

    public LaunchRequest Prepare(GameEntry game, string executable, string? userDirectory)
    {
        if (game.SystemId is not ("gamecube" or "wii")) throw new InvalidOperationException("Ce module prend en charge GameCube et Wii pour le moment.");
        if (string.IsNullOrWhiteSpace(game.FilePath) || !File.Exists(game.FilePath))
            throw new FileNotFoundException("Ajoute un fichier de jeu avant de lancer une partie.");
        if (!File.Exists(executable) || !string.Equals(Path.GetFileName(executable), "Dolphin.exe", StringComparison.OrdinalIgnoreCase))
            throw new FileNotFoundException("Sélectionne Dolphin.exe dans les réglages.");
        // CLI overrides are session-only. Existing Dolphin profiles and saves remain in use.
        List<string> arguments = ["--batch", "--exec", Path.GetFullPath(game.FilePath),
            "--config", "Dolphin.Display.Fullscreen=True",
            "--config", "Dolphin.Interface.ConfirmStop=False"];
        if (!string.IsNullOrWhiteSpace(userDirectory)) arguments.AddRange(["--user", Path.GetFullPath(userDirectory)]);
        return new(Path.GetFullPath(executable), arguments, Path.GetDirectoryName(Path.GetFullPath(executable))!);
    }
}

public interface ISessionRunner
{
    bool IsRunning { get; }
    Task<int> RunAsync(LaunchRequest request);
    bool RequestStop();
}

public sealed class SessionRunner : ISessionRunner
{
    private int running;
    private readonly object processLock = new();
    private Process? activeProcess;
    public bool IsRunning => Volatile.Read(ref running) != 0;

    // Ask the window to close normally. Never terminate a process or target other emulator instances.
    public bool RequestStop()
    {
        lock (processLock)
        {
            if (activeProcess is null) return false;
            try
            {
                activeProcess.Refresh();
                return !activeProcess.HasExited && activeProcess.CloseMainWindow();
            }
            catch (InvalidOperationException) { return false; }
            catch (System.ComponentModel.Win32Exception) { return false; }
        }
    }

    public async Task<int> RunAsync(LaunchRequest request)
    {
        if (Interlocked.CompareExchange(ref running, 1, 0) != 0)
            throw new InvalidOperationException("Une partie est déjà en cours.");
        Process? process = null;
        try
        {
            var start = new ProcessStartInfo(request.Executable) { UseShellExecute = false, WorkingDirectory = request.WorkingDirectory };
            foreach (var argument in request.Arguments) start.ArgumentList.Add(argument);
            process = Process.Start(start) ?? throw new InvalidOperationException("Le jeu n'a pas pu démarrer.");
            lock (processLock) activeProcess = process;
            await process.WaitForExitAsync();
            return process.ExitCode;
        }
        finally
        {
            lock (processLock) { activeProcess = null; process?.Dispose(); }
            Volatile.Write(ref running, 0);
        }
    }
}
