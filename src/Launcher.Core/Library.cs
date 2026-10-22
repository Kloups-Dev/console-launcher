using System.Text.Json;

namespace Launcher.Core;

public sealed record ConsoleSystem(string Id, string Name, string Maker, string Mark, string Description);
public sealed record GameEntry(string Id, string SystemId, string Title, string? FilePath = null);
public sealed class LibraryData
{
    public string? DolphinPath { get; set; }
    public List<GameEntry> Games { get; set; } = [];
}

public static class Catalog
{
    public static IReadOnlyList<ConsoleSystem> Systems { get; } = [
        new("gamecube", "GameCube", "Nintendo · 2001", "GC", "Une sélection de jeux, une manette, et c'est parti."),
        new("ps2", "PlayStation 2", "Sony · 2000", "PS2", "Une autre génération à retrouver, bientôt dans le lanceur."),
        new("snes", "Super Nintendo", "Nintendo · 1990", "16", "Les grands classiques, dans une interface qui va à l'essentiel.")
    ];

    // Original fictional titles: these entries are previews, never downloadable games.
    public static IReadOnlyList<GameEntry> DemoGames { get; } = [
        new("demo-orbit", "gamecube", "Orbit Racers"), new("demo-tide", "gamecube", "Tidebound"),
        new("demo-neon", "ps2", "Neon District"), new("demo-sky", "snes", "Sky Garden")
    ];
}

public sealed class LibraryStore(string path)
{
    private static readonly JsonSerializerOptions Options = new() { WriteIndented = true };

    public LibraryData Load()
    {
        if (!File.Exists(path)) return new();
        var data = JsonSerializer.Deserialize<LibraryData>(File.ReadAllText(path))
            ?? throw new InvalidDataException("La bibliothèque est vide ou invalide.");
        if (data.Games is null || data.Games.Any(g => g is null || string.IsNullOrWhiteSpace(g.Id)
            || string.IsNullOrWhiteSpace(g.SystemId) || string.IsNullOrWhiteSpace(g.Title)))
            throw new InvalidDataException("La bibliothèque contient des entrées invalides.");
        return data;
    }

    public void Save(LibraryData data)
    {
        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(path))!);
        var temporary = path + ".tmp";
        File.WriteAllText(temporary, JsonSerializer.Serialize(data, Options));
        File.Move(temporary, path, overwrite: true);
    }

    public static GameEntry ImportGameCube(LibraryData data, string path)
    {
        var fullPath = Path.GetFullPath(path);
        if (!File.Exists(fullPath)) throw new FileNotFoundException("Le fichier de jeu est introuvable.");
        if (!new[] { ".iso", ".gcm", ".rvz", ".gcz", ".ciso" }.Contains(Path.GetExtension(fullPath).ToLowerInvariant()))
            throw new InvalidDataException("Format non pris en charge pour cette première intégration.");
        var existing = data.Games.FirstOrDefault(g => string.Equals(g.FilePath, fullPath, StringComparison.OrdinalIgnoreCase));
        if (existing is not null) return existing;
        var game = new GameEntry(Guid.NewGuid().ToString("N"), "gamecube", Path.GetFileNameWithoutExtension(fullPath), fullPath);
        data.Games.Add(game);
        return game;
    }
}
