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

public sealed class SessionRunner(IDiagnosticSink? diagnostics = null) : ISessionRunner
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
                var sent = !activeProcess.HasExited && activeProcess.CloseMainWindow();
                diagnostics?.Write("process.stop", "Demande de fermeture envoyée=" + sent);
                return sent;
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
        using var monitorCancellation = new CancellationTokenSource();
        Task monitor = Task.CompletedTask;
        Task output = Task.CompletedTask;
        Task error = Task.CompletedTask;
        var elapsed = Stopwatch.StartNew();
        try
        {
            var start = new ProcessStartInfo(request.Executable) { UseShellExecute = false, WorkingDirectory = request.WorkingDirectory,
                RedirectStandardOutput = diagnostics is not null, RedirectStandardError = diagnostics is not null };
            foreach (var argument in request.Arguments) start.ArgumentList.Add(argument);
            process = Process.Start(start) ?? throw new InvalidOperationException("Le jeu n'a pas pu démarrer.");
            lock (processLock) activeProcess = process;
            if (diagnostics is not null)
            {
                diagnostics.Write("process.start", System.Text.Json.JsonSerializer.Serialize(new { process.Id, request.Executable, request.Arguments, request.WorkingDirectory }));
                output = Capture(process.StandardOutput, "process.stdout", monitorCancellation.Token);
                error = Capture(process.StandardError, "process.stderr", monitorCancellation.Token);
                monitor = Monitor(process, monitorCancellation.Token);
            }
            await process.WaitForExitAsync();
            try { await Task.WhenAll(output, error).WaitAsync(TimeSpan.FromSeconds(2)); }
            catch (TimeoutException) { diagnostics?.Write("process.output", "Fin de collecte : flux encore ouvert après la sortie.", DiagnosticLevel.Observation); }
            diagnostics?.Write("process.exit", $"PID={process.Id}; code={process.ExitCode}; durée={elapsed.Elapsed.TotalSeconds:F2}s",
                process.ExitCode == 0 ? DiagnosticLevel.Info : DiagnosticLevel.Error);
            return process.ExitCode;
        }
        catch (Exception ex)
        { diagnostics?.Write("process.error", ex.ToString(), DiagnosticLevel.Error); throw; }
        finally
        {
            monitorCancellation.Cancel();
            try { await Task.WhenAll(monitor, output, error); } catch (OperationCanceledException) { }
            lock (processLock) { activeProcess = null; process?.Dispose(); }
            Volatile.Write(ref running, 0);
        }
    }

    private async Task Capture(StreamReader reader, string category, CancellationToken cancellation)
    {
        var buffer = new char[2048];
        var total = 0;
        try
        {
            int count;
            while ((count = await reader.ReadAsync(buffer, cancellation)) > 0)
            {
                if (total < 65536) diagnostics?.Write(category, new string(buffer, 0, Math.Min(count, 65536 - total)), DiagnosticLevel.Observation);
                total += count;
                if (total >= 65536) { diagnostics?.Write(category, "Sortie tronquée à 64 K caractères pour cette session.", DiagnosticLevel.Observation); break; }
            }
            // Continue draining so a noisy emulator cannot block on a full pipe.
            if (total >= 65536) while (await reader.ReadAsync(buffer, cancellation) > 0) { }
        }
        catch (OperationCanceledException) when (cancellation.IsCancellationRequested) { }
        catch (IOException ex) { diagnostics?.Write(category, ex.Message, DiagnosticLevel.Observation); }
    }

    private async Task Monitor(Process process, CancellationToken cancellation)
    {
        try
        {
            while (true)
            {
                await Task.Delay(TimeSpan.FromSeconds(10), cancellation);
                process.Refresh();
                if (process.HasExited) return;
                diagnostics?.Write("process.resources", $"PID={process.Id}; mémoire={process.WorkingSet64 / 1048576}Mo; CPU cumulé={process.TotalProcessorTime.TotalMilliseconds:F0}ms. Mesure brute, sans diagnostic automatique de performance.");
            }
        }
        catch (OperationCanceledException) when (cancellation.IsCancellationRequested) { }
        catch (Exception ex) when (ex is InvalidOperationException or System.ComponentModel.Win32Exception)
        { diagnostics?.Write("process.resources", "Mesure indisponible : " + ex.Message, DiagnosticLevel.Observation); }
    }
}
