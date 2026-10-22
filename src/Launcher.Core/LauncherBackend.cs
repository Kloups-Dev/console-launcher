using System.Text.Json;

namespace Launcher.Core;

public enum SessionPhase { Idle, Preparing, Running, StopRequested, Completed, Failed }
public sealed record BackendStatus(SessionPhase Phase, string Message, string? GameId = null)
{
    public bool Busy => Phase is SessionPhase.Preparing or SessionPhase.Running or SessionPhase.StopRequested;
}
public sealed record DiagnosticEntry(DateTimeOffset At, string Message);
public sealed record Preparation(LaunchRequest? Request, string Summary, string? Error)
{
    public bool CanLaunch => Request is not null && Error is null;
}

// This is the UI contract. It depends on neither WPF nor a particular presentation.
public interface ILauncherBackend
{
    event EventHandler? Changed;
    BackendStatus Status { get; }
    bool CanWrite { get; }
    IReadOnlyList<DiagnosticEntry> Diagnostics { get; }
    IReadOnlyList<GameEntry> Games(string systemId);
    EmulatorConfiguration? Configuration(string systemId);
    void Configure(string systemId, EmulatorConfiguration configuration);
    int Import(string systemId, IReadOnlyList<string> paths);
    void Remove(string gameId);
    Preparation Prepare(string gameId);
    Task StartAsync(string gameId);
    bool RequestStop();
}

public sealed class LauncherBackend : ILauncherBackend
{
    private readonly LibraryStore store;
    private readonly ISessionRunner runner;
    private readonly LaunchPlanner planner = new();
    private readonly List<DiagnosticEntry> diagnostics = [];
    private LibraryData data = new();
    private int active;
    public event EventHandler? Changed;
    public BackendStatus Status { get; private set; } = new(SessionPhase.Idle, "Aucune partie en cours.");
    public bool CanWrite { get; private set; } = true;
    public IReadOnlyList<DiagnosticEntry> Diagnostics { get { lock (diagnostics) return diagnostics.ToArray(); } }

    public LauncherBackend(LibraryStore store, ISessionRunner? runner = null)
    {
        this.store = store;
        this.runner = runner ?? new SessionRunner();
        try
        {
            data = store.Load();
            if (!data.Configurations.ContainsKey("gamecube") && !string.IsNullOrWhiteSpace(data.DolphinPath))
                data.Configurations["gamecube"] = new("dolphin", data.DolphinPath);
            Log("Bibliothèque chargée. Les profils avancés et les BIOS ne sont pas validés par cet outil.");
        }
        catch (Exception ex) when (ex is IOException or InvalidDataException or JsonException or UnauthorizedAccessException)
        {
            CanWrite = false;
            Status = new(SessionPhase.Failed, "Bibliothèque illisible : données conservées, écriture bloquée.");
            Log(Status.Message + " " + ex.Message);
        }
    }

    public IReadOnlyList<GameEntry> Games(string systemId) => data.Games.Where(g => g.SystemId == systemId)
        .OrderBy(g => g.Title, StringComparer.CurrentCultureIgnoreCase).ToArray();
    public EmulatorConfiguration? Configuration(string systemId) => data.Configurations.GetValueOrDefault(systemId);

    private void EnsureEditable()
    {
        if (!CanWrite) throw new InvalidOperationException("Bibliothèque illisible : réparez le fichier avant de modifier les données.");
        if (Status.Busy) throw new InvalidOperationException("Attendez la fin de la partie avant de modifier la bibliothèque.");
    }

    // Persist a copy before replacing in-memory state: failed writes never partially apply a change.
    private void Save(LibraryData next)
    {
        store.Save(next);
        data = next;
    }
    private LibraryData Copy() => new() { DolphinPath = data.DolphinPath, Games = [.. data.Games], Configurations = new(data.Configurations) };

    public void Configure(string systemId, EmulatorConfiguration configuration)
    {
        EnsureEditable();
        if (!IntegrationCatalog.Get(systemId).Adapters.Contains(configuration.AdapterId))
            throw new InvalidOperationException("Ce module n'est pas intégré pour cette console.");
        var next = Copy();
        next.Configurations[systemId] = configuration;
        if (systemId == "gamecube" && configuration.AdapterId == "dolphin") next.DolphinPath = configuration.Executable;
        Save(next);
        Log("Configuration enregistrée pour " + IntegrationCatalog.Get(systemId).Name + ". Aucun fichier d'émulateur modifié.");
    }

    public int Import(string systemId, IReadOnlyList<string> paths)
    {
        EnsureEditable();
        var system = IntegrationCatalog.Get(systemId);
        var next = Copy();
        var count = 0;
        foreach (var path in paths)
        {
            var fullPath = Path.GetFullPath(path);
            if (!File.Exists(fullPath)) throw new FileNotFoundException("Fichier introuvable : " + fullPath);
            if (!system.Extensions.Contains(Path.GetExtension(fullPath).ToLowerInvariant()))
                throw new InvalidDataException("Format non prévu pour " + system.Name + " : " + Path.GetExtension(fullPath));
            // A duplicate path is idempotent within its system; consoles sharing .iso stay explicit.
            if (next.Games.Any(g => g.SystemId == systemId && string.Equals(g.FilePath, fullPath, StringComparison.OrdinalIgnoreCase))) continue;
            next.Games.Add(new(Guid.NewGuid().ToString("N"), systemId, Path.GetFileNameWithoutExtension(fullPath), fullPath));
            count++;
        }
        Save(next);
        Log($"{count} jeu(x) ajouté(s) à {system.Name}. Les fichiers restent à leur emplacement d'origine.");
        return count;
    }

    public void Remove(string gameId)
    {
        EnsureEditable();
        var next = Copy();
        var count = next.Games.RemoveAll(g => g.Id == gameId);
        if (count == 0) return;
        Save(next);
        Log("Entrée retirée de la bibliothèque. Aucun fichier de jeu ni sauvegarde supprimé.");
    }

    public Preparation Prepare(string gameId)
    {
        try
        {
            var game = data.Games.FirstOrDefault(g => g.Id == gameId) ?? throw new InvalidOperationException("Choisissez un jeu de la bibliothèque.");
            if (IntegrationCatalog.Get(game.SystemId).Adapters.Length == 0)
                throw new InvalidOperationException("Cette console est cataloguée, mais son module n'est pas encore intégré.");
            var configuration = Configuration(game.SystemId) ?? throw new InvalidOperationException("Enregistrez l'émulateur et, pour RetroArch, son cœur.");
            var request = planner.Prepare(game, configuration);
            return new(request, "Commande prête. Les réglages personnels restent ceux de l'émulateur. BIOS, contenu et compatibilité à vérifier au lancement.", null);
        }
        catch (Exception ex) when (ex is IOException or InvalidDataException or InvalidOperationException or ArgumentException or UnauthorizedAccessException)
        { return new(null, "Préparation bloquée.", ex.Message); }
    }

    public async Task StartAsync(string gameId)
    {
        if (Interlocked.CompareExchange(ref active, 1, 0) != 0)
            throw new InvalidOperationException("Une préparation ou une partie est déjà en cours.");
        try
        {
            SetStatus(SessionPhase.Preparing, "Vérification des prérequis…", gameId);
            var preparation = Prepare(gameId);
            if (!preparation.CanLaunch) { SetStatus(SessionPhase.Failed, preparation.Error!, gameId); return; }
            // Start synchronously before publishing Running; prevents a stop targeting no process yet.
            var completion = runner.RunAsync(preparation.Request!);
            if (completion.IsFaulted) await completion;
            SetStatus(SessionPhase.Running, "Partie démarrée. Demande de fermeture disponible.", gameId);
            var exit = await completion;
            SetStatus(exit == 0 ? SessionPhase.Completed : SessionPhase.Failed,
                exit == 0 ? "Partie terminée. Bibliothèque disponible." : $"L'émulateur s'est arrêté avec le code {exit}.", gameId);
        }
        catch (Exception ex) when (ex is IOException or InvalidOperationException or System.ComponentModel.Win32Exception or UnauthorizedAccessException)
        { SetStatus(SessionPhase.Failed, "Lancement impossible : " + ex.Message, gameId); }
        finally { Volatile.Write(ref active, 0); }
    }

    public bool RequestStop()
    {
        if (Status.Phase is not (SessionPhase.Running or SessionPhase.StopRequested)) return false;
        if (!runner.RequestStop()) { Log("Fermeture non envoyée : fenêtre pas encore prête ou dialogue ouvert. Réessayez ou fermez le jeu normalement."); return false; }
        SetStatus(SessionPhase.StopRequested, "Fermeture normale demandée. En attente de l'émulateur…", Status.GameId);
        return true;
    }

    private void SetStatus(SessionPhase phase, string message, string? gameId)
    {
        Status = new(phase, message, gameId);
        Log(message);
    }
    private void Log(string message)
    {
        lock (diagnostics)
        {
            diagnostics.Add(new(DateTimeOffset.Now, message));
            if (diagnostics.Count > 150) diagnostics.RemoveAt(0);
        }
        Changed?.Invoke(this, EventArgs.Empty);
    }
}
