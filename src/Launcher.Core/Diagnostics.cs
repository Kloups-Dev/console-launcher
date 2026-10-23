using System.Text.Json;

namespace Launcher.Core;

public enum DiagnosticLevel { Info, Observation, Error }
public interface IDiagnosticSink
{
    void Write(string category, string message, DiagnosticLevel level = DiagnosticLevel.Info);
}

public sealed class DiagnosticLog : IDiagnosticSink
{
    private readonly object sync = new();
    private readonly long limit;
    private readonly string redactPath;
    public string DirectoryPath { get; }
    public string FilePath => Path.Combine(DirectoryPath, "launcher.jsonl");
    public string? LastWriteError { get; private set; }
    public string RunId { get; } = Guid.NewGuid().ToString("N");

    public DiagnosticLog(string directory, long sizeLimit = 512 * 1024, string? pathToRedact = null)
    {
        DirectoryPath = Path.GetFullPath(directory);
        limit = Math.Max(1024, sizeLimit);
        redactPath = pathToRedact ?? Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
    }

    public void Write(string category, string message, DiagnosticLevel level = DiagnosticLevel.Info)
    {
        lock (sync)
        {
            try
            {
                Directory.CreateDirectory(DirectoryPath);
                if (File.Exists(FilePath) && new FileInfo(FilePath).Length >= limit)
                {
                    // Five bounded files; names never come from logged content.
                    for (var i = 3; i >= 1; i--)
                        if (File.Exists(FilePath + "." + i)) File.Move(FilePath + "." + i, FilePath + "." + (i + 1), true);
                    File.Move(FilePath, FilePath + ".1", true);
                }
                var safe = message.Length > 4096 ? message[..4096] + " [tronqué]" : message;
                if (!string.IsNullOrEmpty(redactPath))
                {
                    safe = safe.Replace(redactPath, "%USERPROFILE%", StringComparison.OrdinalIgnoreCase)
                        .Replace(redactPath.Replace('\\', '/'), "%USERPROFILE%", StringComparison.OrdinalIgnoreCase)
                        .Replace(redactPath.Replace("\\", "\\\\"), "%USERPROFILE%", StringComparison.OrdinalIgnoreCase);
                }
                var json = JsonSerializer.Serialize(new { At = DateTimeOffset.Now, RunId, Level = level.ToString(), Category = category, Message = safe });
                File.AppendAllText(FilePath, json + Environment.NewLine);
                LastWriteError = null;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            { LastWriteError = ex.Message; } // Logging must never break a game or hide the original failure.
        }
    }

    public string[] Snapshot()
    {
        lock (sync)
        {
            var lines = new List<string>();
            for (var i = 4; i >= 0; i--)
            {
                var path = i == 0 ? FilePath : FilePath + "." + i;
                if (File.Exists(path)) lines.AddRange(File.ReadAllLines(path));
            }
            return lines.ToArray();
        }
    }
}

// Counts only activity in the explicit test surface. Never records text or global keystrokes.
public sealed class InputProbe
{
    public bool Active { get; private set; }
    public int KeyDowns { get; private set; }
    public int KeyUps { get; private set; }
    public int MouseMoves { get; private set; }
    public int MouseClicks { get; private set; }
    public int FocusLosses { get; private set; }
    public void Start() { KeyDowns = KeyUps = MouseMoves = MouseClicks = FocusLosses = 0; Active = true; }
    public void Key(bool down, bool repeated = false) { if (!Active || repeated) return; if (down) KeyDowns++; else KeyUps++; }
    public void Mouse(bool click) { if (!Active) return; if (click) MouseClicks++; else MouseMoves++; }
    public void LoseFocus() { if (Active) FocusLosses++; }
    public string Summary(bool remote) => $"Test local : touches appuyées={KeyDowns}, relâchées={KeyUps}, mouvements souris={MouseMoves}, clics={MouseClicks}, pertes de focus={FocusLosses}. "
        + (remote ? "Contexte distant déclaré/détecté. " : "Contexte distant non signalé. ")
        + "Les différences de comptage ou l'absence d'événements ne prouvent pas un bug. Ce test ne mesure pas les entrées reçues par Dolphin.";
    public void Stop() => Active = false;
}
