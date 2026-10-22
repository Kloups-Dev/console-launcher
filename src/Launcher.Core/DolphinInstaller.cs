using System.Security.Cryptography;
using System.Text.Json;
using SharpCompress.Archives.SevenZip;

namespace Launcher.Core;

// Curated release: changing the version requires checking the official archive and its hash again.
public sealed record DolphinPackage(string Version, Uri Url, string Sha256)
{
    public static DolphinPackage Current { get; } = new("2609a",
        new("https://dl.dolphin-emu.org/releases/2609a/dolphin-2609a-x64.7z"),
        "BBFD13BF9E6D2A4164A15C392B1F32A9B18150E5136746B388704608FBB60F76");
}

public sealed record DolphinInstallation(string Version, string Executable, string UserDirectory, string ExecutableHash);
public sealed record InstallProgress(string Message, int? Percent = null);

public sealed class DolphinInstaller(HttpClient http, string dataDirectory)
{
    private readonly string root = Path.GetFullPath(Path.Combine(dataDirectory, "emulators", "dolphin"));
    private readonly SemaphoreSlim gate = new(1, 1);
    public string UserDirectory => Path.Combine(root, "user");

    public DolphinInstallation? Detect()
    {
        try
        {
            var path = Path.Combine(root, DolphinPackage.Current.Version, "installation.json");
            if (!File.Exists(path)) return null;
            var found = JsonSerializer.Deserialize<DolphinInstallation>(File.ReadAllText(path));
            var expected = Path.Combine(root, DolphinPackage.Current.Version, "Dolphin-x64", "Dolphin.exe");
            if (found is null || found.Version != DolphinPackage.Current.Version
                || !string.Equals(found.Executable, expected, StringComparison.OrdinalIgnoreCase)
                || found.UserDirectory != UserDirectory || !File.Exists(expected)
                || !File.Exists(Path.Combine(Path.GetDirectoryName(expected)!, "QtPlugins", "platforms", "qwindows.dll"))) return null;
            using var file = File.OpenRead(expected);
            return Convert.ToHexString(SHA256.HashData(file)) == found.ExecutableHash ? found : null;
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or JsonException) { return null; }
    }

    public async Task<DolphinInstallation> InstallAsync(IProgress<InstallProgress>? progress = null,
        CancellationToken cancellation = default)
    {
        if (!await gate.WaitAsync(0, cancellation)) throw new InvalidOperationException("Une installation est déjà en cours.");
        string? staging = null;
        try
        {
            if (Detect() is { } existing)
            {
                InitializeUserDirectory(UserDirectory);
                progress?.Report(new("Dolphin déjà installé et vérifié.", 100));
                return existing;
            }
            Directory.CreateDirectory(root);
            staging = Path.Combine(root, ".install-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(staging);
            var archive = Path.Combine(staging, "download.7z");
            progress?.Report(new("Téléchargement de Dolphin " + DolphinPackage.Current.Version + " depuis le site officiel…"));
            await DownloadVerifiedAsync(http, DolphinPackage.Current, archive, progress, cancellation);
            progress?.Report(new("Archive vérifiée. Extraction…"));
            var extracted = Path.Combine(staging, "files");
            Directory.CreateDirectory(extracted);
            await Task.Run(() => ExtractArchive(archive, extracted, cancellation), cancellation);
            var executable = Path.Combine(extracted, "Dolphin-x64", "Dolphin.exe");
            if (!File.Exists(executable) || !File.Exists(Path.Combine(extracted, "Dolphin-x64", "QtPlugins", "platforms", "qwindows.dll")))
                throw new InvalidDataException("L'archive ne contient pas une installation Dolphin complète.");
            using (var file = File.OpenRead(executable))
                if (file.ReadByte() != 'M' || file.ReadByte() != 'Z') throw new InvalidDataException("Exécutable Dolphin invalide.");
            cancellation.ThrowIfCancellationRequested();
            var destination = Path.Combine(root, DolphinPackage.Current.Version);
            // Keep even damaged previous directories: no overwrite of running files or user data.
            if (Directory.Exists(destination)) Directory.Move(destination, destination + ".previous-" + Guid.NewGuid().ToString("N"));
            Directory.Move(extracted, destination);
            executable = Path.Combine(destination, "Dolphin-x64", "Dolphin.exe");
            string hash;
            using (var file = File.OpenRead(executable)) hash = Convert.ToHexString(SHA256.HashData(file));
            var installed = new DolphinInstallation(DolphinPackage.Current.Version, executable, UserDirectory, hash);
            InitializeUserDirectory(UserDirectory);
            File.WriteAllText(Path.Combine(destination, "installation.json"), JsonSerializer.Serialize(installed));
            progress?.Report(new("Dolphin installé. Profil GameCube XInput de départ prêt.", 100));
            return installed;
        }
        finally
        {
            // Only our unique staging directory, never an emulator/profile directory.
            try { if (staging is not null && Directory.Exists(staging)) Directory.Delete(staging, true); }
            finally { gate.Release(); }
        }
    }

    public static async Task DownloadVerifiedAsync(HttpClient http, DolphinPackage package, string destination,
        IProgress<InstallProgress>? progress, CancellationToken cancellation)
    {
        if (package.Url.Scheme != "https" || package.Url.Host != "dl.dolphin-emu.org")
            throw new InvalidDataException("Source Dolphin non autorisée.");
        using var response = await http.GetAsync(package.Url, HttpCompletionOption.ResponseHeadersRead, cancellation);
        response.EnsureSuccessStatusCode();
        var actual = response.RequestMessage?.RequestUri;
        if (actual is not null && (actual.Scheme != "https" || actual.Host != "dl.dolphin-emu.org"))
            throw new InvalidDataException("Redirection inattendue du téléchargement.");
        const long limit = 256 * 1024 * 1024;
        if (response.Content.Headers.ContentLength > limit) throw new InvalidDataException("Archive trop volumineuse.");
        await using var source = await response.Content.ReadAsStreamAsync(cancellation);
        await using var output = new FileStream(destination, FileMode.CreateNew, FileAccess.Write, FileShare.None, 81920, true);
        using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
        var buffer = new byte[81920];
        long total = 0;
        int lastPercent = -1;
        while (true)
        {
            var count = await source.ReadAsync(buffer, cancellation);
            if (count == 0) break;
            total += count;
            if (total > limit) throw new InvalidDataException("Archive trop volumineuse.");
            hash.AppendData(buffer, 0, count);
            await output.WriteAsync(buffer.AsMemory(0, count), cancellation);
            if (response.Content.Headers.ContentLength is > 0 and var size)
            {
                var percent = (int)Math.Min(100, total * 100 / size);
                if (percent != lastPercent) { lastPercent = percent; progress?.Report(new("Téléchargement de Dolphin…", percent)); }
            }
        }
        if (!string.Equals(Convert.ToHexString(hash.GetHashAndReset()), package.Sha256, StringComparison.OrdinalIgnoreCase))
            throw new InvalidDataException("Archive Dolphin différente de la version vérifiée. Installation annulée.");
    }

    public static string SafeDestination(string directory, string entry)
    {
        var normalized = entry.Replace('\\', '/');
        if (string.IsNullOrWhiteSpace(normalized) || normalized.StartsWith('/') || normalized.Contains(':')
            || normalized.Split('/').Any(part => part is ".." or ".")) throw new InvalidDataException("Chemin d'archive interdit.");
        var prefix = Path.GetFullPath(directory) + Path.DirectorySeparatorChar;
        var full = Path.GetFullPath(Path.Combine(directory, normalized));
        if (!full.StartsWith(prefix, StringComparison.OrdinalIgnoreCase)) throw new InvalidDataException("Chemin hors du dossier d'installation.");
        return full;
    }

    public static void ExtractArchive(string archivePath, string destination, CancellationToken cancellation)
    {
        using var archive = SevenZipArchive.OpenArchive(archivePath);
        using var reader = archive.ExtractAllEntries();
        long total = 0;
        var entries = 0;
        while (reader.MoveToNextEntry())
        {
            cancellation.ThrowIfCancellationRequested();
            if (++entries > 20000 || reader.Entry.LinkTarget is not null) throw new InvalidDataException("Archive non prise en charge.");
            var path = SafeDestination(destination, reader.Entry.Key ?? "");
            if (reader.Entry.IsDirectory) { Directory.CreateDirectory(path); continue; }
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            using var source = reader.OpenEntryStream();
            using var output = new FileStream(path, FileMode.CreateNew);
            var buffer = new byte[81920];
            int count;
            while ((count = source.Read(buffer)) > 0)
            {
                cancellation.ThrowIfCancellationRequested();
                total += count;
                if (total > 1024L * 1024 * 1024) throw new InvalidDataException("Archive décompressée trop volumineuse.");
                output.Write(buffer, 0, count);
            }
        }
    }

    public static void InitializeUserDirectory(string directory)
    {
        var config = Path.Combine(directory, "Config");
        Directory.CreateDirectory(config);
        var path = Path.Combine(config, "GCPadNew.ini");
        // First-run profile only. Never overwrite the player's changes.
        if (!File.Exists(path)) File.WriteAllText(path, """
            [GCPad1]
            Device = XInput/0/Gamepad
            Buttons/A = `Button A`
            Buttons/B = `Button B`
            Buttons/X = `Button X`
            Buttons/Y = `Button Y`
            Buttons/Z = `Shoulder R`
            Buttons/Start = Start
            Main Stick/Up = `Left Y+`
            Main Stick/Down = `Left Y-`
            Main Stick/Left = `Left X-`
            Main Stick/Right = `Left X+`
            Main Stick/Dead Zone = 15
            C-Stick/Up = `Right Y+`
            C-Stick/Down = `Right Y-`
            C-Stick/Left = `Right X-`
            C-Stick/Right = `Right X+`
            C-Stick/Dead Zone = 15
            Triggers/L = `Trigger L`
            Triggers/R = `Trigger R`
            Triggers/L-Analog = `Trigger L`
            Triggers/R-Analog = `Trigger R`
            D-Pad/Up = `Pad N`
            D-Pad/Down = `Pad S`
            D-Pad/Left = `Pad W`
            D-Pad/Right = `Pad E`
            """);
    }
}
