namespace Launcher.Core;

public sealed record SystemDefinition(string Id, string Name, string Family, string[] Adapters, string[] Extensions)
{
    public string Integration => Adapters.Length > 0 ? "Module disponible · à configurer" : "Cataloguée · module à développer";
}
public sealed record EmulatorConfiguration(string AdapterId, string Executable, string? CorePath = null, string? UserDirectory = null);

public static class IntegrationCatalog
{
    private static readonly string[] Discs = [".iso", ".bin", ".cue", ".chd", ".m3u"];
    public static IReadOnlyList<SystemDefinition> Systems { get; } = [
        new("gamecube", "GameCube", "Nintendo", ["dolphin"], [".iso", ".gcm", ".rvz", ".gcz", ".ciso"]),
        new("wii", "Wii", "Nintendo", ["dolphin"], [".iso", ".wbfs", ".rvz", ".gcz", ".wad"]),
        new("ps2", "PlayStation 2", "PlayStation", ["pcsx2"], [".iso", ".bin", ".chd", ".cso", ".gz", ".m3u"]),
        new("nes", "NES", "Nintendo", ["retroarch"], [".nes", ".fds", ".zip", ".7z"]),
        new("snes", "Super Nintendo", "Nintendo", ["retroarch"], [".sfc", ".smc", ".zip", ".7z"]),
        new("n64", "Nintendo 64", "Nintendo", ["retroarch"], [".n64", ".z64", ".v64", ".zip"]),
        new("gb", "Game Boy", "Nintendo", ["retroarch"], [".gb", ".zip"]),
        new("gbc", "Game Boy Color", "Nintendo", ["retroarch"], [".gbc", ".gb", ".zip"]),
        new("gba", "Game Boy Advance", "Nintendo", ["retroarch"], [".gba", ".zip", ".7z"]),
        new("nds", "Nintendo DS", "Nintendo", ["retroarch"], [".nds", ".zip", ".7z"]),
        new("3ds", "Nintendo 3DS", "Nintendo", [], [".3ds", ".cci", ".cia"]),
        new("wiiu", "Wii U", "Nintendo", [], [".wud", ".wux", ".wua", ".rpx"]),
        new("switch", "Nintendo Switch", "Nintendo", [], [".nsp", ".xci", ".nro"]),
        new("ps1", "PlayStation", "PlayStation", ["retroarch"], Discs),
        new("psp", "PSP", "PlayStation", ["retroarch"], [".iso", ".cso", ".pbp"]),
        new("ps3", "PlayStation 3", "PlayStation", [], [".pkg", ".elf", ".self"]),
        new("vita", "PS Vita", "PlayStation", [], [".vpk", ".pkg"]),
        new("ps4", "PlayStation 4", "PlayStation", [], [".pkg", ".elf"]),
        new("ps5", "PlayStation 5", "PlayStation", [], []),
        new("xbox", "Xbox", "Xbox", [], [".iso", ".xbe"]),
        new("xbox360", "Xbox 360", "Xbox", [], [".iso", ".xex"]),
        new("xboxone", "Xbox One", "Xbox", [], []),
        new("xboxseries", "Xbox Series", "Xbox", [], []),
        new("mastersystem", "Master System", "Sega", ["retroarch"], [".sms", ".zip"]),
        new("megadrive", "Mega Drive", "Sega", ["retroarch"], [".md", ".gen", ".smd", ".bin", ".zip"]),
        new("segacd", "Mega-CD", "Sega", ["retroarch"], Discs),
        new("32x", "Sega 32X", "Sega", ["retroarch"], [".32x", ".bin", ".zip"]),
        new("gamegear", "Game Gear", "Sega", ["retroarch"], [".gg", ".zip"]),
        new("saturn", "Saturn", "Sega", ["retroarch"], Discs),
        new("dreamcast", "Dreamcast", "Sega", ["retroarch"], [".gdi", ".cdi", ".chd", ".m3u"]),
        new("pcengine", "PC Engine", "NEC", ["retroarch"], [".pce", ".cue", ".chd", ".zip"]),
        new("neogeo", "Neo Geo", "SNK", ["retroarch"], [".zip", ".7z"]),
        new("ngp", "Neo Geo Pocket", "SNK", ["retroarch"], [".ngp", ".ngc", ".zip"]),
        new("atari2600", "Atari 2600", "Atari", ["retroarch"], [".a26", ".bin", ".zip"]),
        new("lynx", "Atari Lynx", "Atari", ["retroarch"], [".lnx", ".zip"]),
        new("arcade", "Arcade", "Autres", ["retroarch"], [".zip", ".7z"])
    ];

    public static SystemDefinition Get(string id) => Systems.FirstOrDefault(s => s.Id == id)
        ?? throw new InvalidOperationException("Console inconnue : " + id);
}

public sealed class LaunchPlanner
{
    public LaunchRequest Prepare(GameEntry game, EmulatorConfiguration configuration)
    {
        var system = IntegrationCatalog.Get(game.SystemId);
        if (!system.Adapters.Contains(configuration.AdapterId))
            throw new InvalidOperationException("Ce module ne peut pas lancer cette console dans cette version.");
        if (string.IsNullOrWhiteSpace(game.FilePath) || !File.Exists(game.FilePath))
            throw new FileNotFoundException("Le fichier du jeu est absent. Ajoutez ou retrouvez son fichier.");
        if (!system.Extensions.Contains(Path.GetExtension(game.FilePath).ToLowerInvariant()))
            throw new InvalidDataException("Ce format de fichier n'est pas prévu pour cette console.");
        if (configuration.AdapterId == "dolphin") return new DolphinAdapter().Prepare(game, configuration.Executable, configuration.UserDirectory);
        if (!File.Exists(configuration.Executable)) throw new FileNotFoundException("L'exécutable de l'émulateur est absent.");
        var executable = Path.GetFullPath(configuration.Executable);
        var content = Path.GetFullPath(game.FilePath);
        var workingDirectory = Path.GetDirectoryName(executable)!;
        if (configuration.AdapterId == "pcsx2")
        {
            if (!new[] { "pcsx2.exe", "pcsx2-qt.exe" }.Contains(Path.GetFileName(executable).ToLowerInvariant()))
                throw new InvalidDataException("Sélectionnez pcsx2.exe ou pcsx2-qt.exe.");
            return new(executable, ["-batch", "-nogui", "-fullscreen", "--", content], workingDirectory);
        }
        if (!string.Equals(Path.GetFileName(executable), "retroarch.exe", StringComparison.OrdinalIgnoreCase))
            throw new InvalidDataException("Sélectionnez retroarch.exe.");
        if (string.IsNullOrWhiteSpace(configuration.CorePath) || !File.Exists(configuration.CorePath)
            || !string.Equals(Path.GetExtension(configuration.CorePath), ".dll", StringComparison.OrdinalIgnoreCase))
            throw new FileNotFoundException("Sélectionnez le cœur RetroArch Windows (.dll) adapté à cette console.");
        return new(executable, ["-L", Path.GetFullPath(configuration.CorePath), "--fullscreen", content], workingDirectory);
    }
}
