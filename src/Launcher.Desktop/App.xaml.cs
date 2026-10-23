using System.Windows;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using Launcher.Core;
namespace Launcher.Desktop;
public partial class App : Application
{
    public static DiagnosticLog Diagnostics { get; private set; } = new(Path.Combine(Path.GetTempPath(), "ConsoleLauncher-logs"));
    public static bool RemoteDetected => SystemParameters.IsRemoteSession;
    public static bool RemoteDeclared { get; private set; }
    public static bool RemoteContext => RemoteDetected || RemoteDeclared;
    private static string settingsPath = "";

    protected override void OnStartup(StartupEventArgs e)
    {
        var index = Array.IndexOf(e.Args, "--data-dir");
        var directory = index >= 0 && index + 1 < e.Args.Length ? e.Args[index + 1]
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "ConsoleLauncher");
        Diagnostics = new(Path.Combine(directory, "logs"));
        settingsPath = Path.Combine(directory, "remote-input-test.txt");
        try
        {
            RemoteDeclared = File.Exists(settingsPath) ? File.ReadAllText(settingsPath).Trim() == "true"
                : File.Exists(Path.Combine(AppContext.BaseDirectory, "remote-input-test.flag"));
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        { Diagnostics.Write("environment.read", ex.Message, DiagnosticLevel.Observation); }
        DispatcherUnhandledException += (_, args) => Diagnostics.Write("crash.ui", args.Exception.ToString(), DiagnosticLevel.Error);
        AppDomain.CurrentDomain.UnhandledException += (_, args) => Diagnostics.Write("crash.runtime", args.ExceptionObject.ToString() ?? "Unknown", DiagnosticLevel.Error);
        TaskScheduler.UnobservedTaskException += (_, args) => Diagnostics.Write("task.unobserved", args.Exception.ToString(), DiagnosticLevel.Error);
        Diagnostics.Write("app.start", $"Version={Assembly.GetExecutingAssembly().GetCustomAttribute<AssemblyInformationalVersionAttribute>()?.InformationalVersion}; OS={RuntimeInformation.OSDescription}; architecture={RuntimeInformation.OSArchitecture}; runtime={RuntimeInformation.FrameworkDescription}; écrans={SystemParameters.VirtualScreenWidth}x{SystemParameters.VirtualScreenHeight}; rendu WPF={System.Windows.Media.RenderCapability.Tier >> 16}");
        LogRemoteContext();
        base.OnStartup(e);
    }

    public static void SetRemoteDeclared(bool value)
    {
        RemoteDeclared = value;
        try { Directory.CreateDirectory(Path.GetDirectoryName(settingsPath)!); File.WriteAllText(settingsPath, value ? "true" : "false"); }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        { Diagnostics.Write("environment.save", ex.Message, DiagnosticLevel.Error); }
        LogRemoteContext();
    }
    private static void LogRemoteContext() => Diagnostics.Write("environment.remote",
        $"Session distante Windows détectée={RemoteDetected}; test distant déclaré={RemoteDeclared}. Les entrées clavier/souris absentes ou différentes ne sont pas automatiquement des erreurs.", DiagnosticLevel.Observation);
    protected override void OnExit(ExitEventArgs e) { Diagnostics.Write("app.exit", "Code=" + e.ApplicationExitCode); base.OnExit(e); }
}
