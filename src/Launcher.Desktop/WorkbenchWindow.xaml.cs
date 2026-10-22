using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using Launcher.Core;
using Microsoft.Win32;

namespace Launcher.Desktop;

public partial class WorkbenchWindow : Window
{
    private readonly ILauncherBackend backend;
    private readonly string dataDirectory;
    private readonly Gamepad gamepad = new();
    private readonly DispatcherTimer sessionInput = new() { Interval = TimeSpan.FromMilliseconds(50) };
    private bool rendering;
    private static readonly HttpClient downloadClient = new() { Timeout = TimeSpan.FromMinutes(5) };
    private readonly DolphinInstaller dolphinInstaller;
    private DolphinInstallation? managedDolphin;
    private CancellationTokenSource? installationCancellation;
    private bool Installing => installationCancellation is not null;
    private string? previewPath;
    private SystemDefinition? SelectedSystem => SystemsList.SelectedItem as SystemDefinition;
    private GameEntry? SelectedGame => GamesGrid.SelectedItem as GameEntry;

    public WorkbenchWindow()
    {
        InitializeComponent();
        var arguments = Environment.GetCommandLineArgs();
        var index = Array.IndexOf(arguments, "--data-dir");
        dataDirectory = index >= 0 && index + 1 < arguments.Length ? arguments[index + 1]
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "ConsoleLauncher");
        index = Array.IndexOf(arguments, "--preview");
        if (index >= 0 && index + 1 < arguments.Length) previewPath = arguments[index + 1];
        backend = new LauncherBackend(new LibraryStore(Path.Combine(dataDirectory, "library.json")));
        dolphinInstaller = new(downloadClient, dataDirectory);
        managedDolphin = dolphinInstaller.Detect();
        backend.Changed += OnBackendChanged;
        SystemsList.ItemsSource = IntegrationCatalog.Systems;
        SystemsList.SelectedIndex = 0;
        sessionInput.Tick += (_, _) =>
        {
            gamepad.Poll(false, backend.Status.Busy);
            if (gamepad.ReturnRequested) backend.RequestStop();
        };
        Closing += (_, e) => { if (backend.Status.Busy || Installing) { e.Cancel = true; StatusText.Text = "Terminez la partie ou annulez l'installation avant de quitter l'atelier."; } };
        Closed += (_, _) => { sessionInput.Stop(); backend.Changed -= OnBackendChanged; };
        Loaded += OnLoaded;
        RefreshState();
    }

    private void OnBackendChanged(object? sender, EventArgs e)
    {
        if (Dispatcher.CheckAccess()) RefreshState();
        else Dispatcher.BeginInvoke(new Action(RefreshState));
    }

    private void RefreshState()
    {
        var busy = backend.Status.Busy || Installing;
        var system = SelectedSystem;
        ImportButton.IsEnabled = !busy && backend.CanWrite && system?.Extensions.Length > 0;
        RemoveButton.IsEnabled = !busy && backend.CanWrite && SelectedGame is not null;
        SaveConfigurationButton.IsEnabled = !busy && backend.CanWrite && system?.Adapters.Length > 0;
        AdapterBox.IsEnabled = !busy && system?.Adapters.Length > 0;
        ExecutableBox.IsEnabled = CoreBox.IsEnabled = !busy;
        PrepareButton.IsEnabled = !busy && SelectedGame is not null;
        StartButton.IsEnabled = !busy && SelectedGame is not null && system?.Adapters.Length > 0;
        StopButton.IsEnabled = backend.Status.Phase is SessionPhase.Running or SessionPhase.StopRequested;
        SystemsList.IsEnabled = !Installing;
        DolphinInstallPanel.Visibility = system?.Id is "gamecube" or "wii" ? Visibility.Visible : Visibility.Collapsed;
        InstallDolphinButton.IsEnabled = !busy && backend.CanWrite && RuntimeInformation.OSArchitecture == Architecture.X64;
        InstallDolphinButton.Content = managedDolphin is null ? "Installer Dolphin " + DolphinPackage.Current.Version : "Utiliser Dolphin installé";
        CancelInstallButton.IsEnabled = Installing;
        if (!Installing) DolphinInstallText.Text = managedDolphin is null
            ? "Dolphin n'est pas installé dans le lanceur. Téléchargement officiel et extraction automatique (Windows x64)."
            : "Dolphin " + managedDolphin.Version + " installé et vérifié. Cliquez pour l'utiliser pour cette console.";
        DolphinRuntimeText.Text = new[] { "vcruntime140.dll", "vcruntime140_1.dll", "msvcp140.dll" }
            .All(name => File.Exists(Path.Combine(Environment.SystemDirectory, name)))
            ? "Bibliothèques Visual C++ détectées. Leur compatibilité sera confirmée au démarrage de Dolphin."
            : "Prérequis Microsoft Visual C++ manquant : installez la version x64 depuis le lien ci-dessous avant de jouer.";
        SessionPhaseText.Text = backend.Status.Phase switch
        {
            SessionPhase.Idle => "Disponible", SessionPhase.Preparing => "Préparation", SessionPhase.Running => "En partie",
            SessionPhase.StopRequested => "Fermeture demandée", SessionPhase.Completed => "Terminé", _ => "Action bloquée ou erreur"
        };
        SessionMessageText.Text = backend.Status.Message;
        StatusText.Text = SessionPhaseText.Text + " — " + backend.Status.Message;
        DiagnosticsList.ItemsSource = backend.Diagnostics;
        if (DiagnosticsList.Items.Count > 0) DiagnosticsList.ScrollIntoView(DiagnosticsList.Items[^1]);
    }

    private void OnSystemChanged(object sender, SelectionChangedEventArgs e)
    {
        if (backend is null || SelectedSystem is not { } system) return;
        rendering = true;
        try
        {
            SystemName.Text = system.Name;
            IntegrationText.Text = system.Integration;
            var configuration = backend.Configuration(system.Id);
            AdapterBox.ItemsSource = system.Adapters;
            AdapterBox.SelectedItem = configuration?.AdapterId ?? system.Adapters.FirstOrDefault();
            ExecutableBox.Text = configuration?.Executable ?? "";
            CoreBox.Text = configuration?.CorePath ?? "";
            SearchBox.Text = "";
            PreparationText.Text = "Sélectionnez ou ajoutez un jeu, puis vérifiez sa préparation.";
            PlanText.Text = "";
        }
        finally { rendering = false; }
        RefreshGames();
        UpdateCoreVisibility();
        RefreshState();
    }

    private void RefreshGames(string? selectedId = null)
    {
        if (SelectedSystem is not { } system) return;
        selectedId ??= SelectedGame?.Id;
        var filter = SearchBox.Text.Trim();
        var games = backend.Games(system.Id).Where(g => g.Title.Contains(filter, StringComparison.CurrentCultureIgnoreCase)).ToArray();
        GamesGrid.ItemsSource = games;
        GamesGrid.SelectedItem = games.FirstOrDefault(g => g.Id == selectedId) ?? games.FirstOrDefault();
    }
    private void OnSearchChanged(object sender, TextChangedEventArgs e) { if (!rendering && backend is not null) RefreshGames(); }
    private void OnGameChanged(object sender, SelectionChangedEventArgs e)
    {
        if (backend is null) return;
        PlanText.Text = "";
        PreparationText.Text = SelectedGame is null ? "Aucun jeu dans cette sélection. Ajoutez vos fichiers locaux."
            : "Préparation à vérifier. Les réglages modifiés doivent être enregistrés avant le lancement.";
        RefreshState();
    }
    private void UpdateCoreVisibility() => CorePanel.Visibility = (string?)AdapterBox.SelectedItem == "retroarch" ? Visibility.Visible : Visibility.Collapsed;
    private void OnAdapterChanged(object sender, SelectionChangedEventArgs e) => UpdateCoreVisibility();

    private void Guard(Action action)
    {
        try { if (Installing) throw new InvalidOperationException("Attendez la fin de l'installation ou annulez-la."); action(); }
        catch (Exception ex) when (ex is IOException or InvalidDataException or InvalidOperationException or UnauthorizedAccessException or ArgumentException or System.ComponentModel.Win32Exception)
        { StatusText.Text = ex.Message; MessageBox.Show(this, ex.Message, "Action impossible", MessageBoxButton.OK, MessageBoxImage.Information); }
    }
    private void OnSaveConfiguration(object sender, RoutedEventArgs e) => Guard(() =>
    {
        if (SelectedSystem is not { } system || AdapterBox.SelectedItem is not string adapter) return;
        var executable = ExecutableBox.Text.Trim().Trim('"');
        var userDirectory = managedDolphin is not null && string.Equals(executable, managedDolphin.Executable, StringComparison.OrdinalIgnoreCase)
            ? managedDolphin.UserDirectory : null;
        backend.Configure(system.Id, new(adapter, executable, CoreBox.Text.Trim().Trim('"'), userDirectory));
        PlanText.Text = "";
        PreparationText.Text = "Configuration enregistrée. Vérifiez la préparation du jeu.";
    });
    private void OnBrowseExecutable(object sender, RoutedEventArgs e)
    {
        if (backend.Status.Busy || Installing) return;
        var picker = new OpenFileDialog { Title = "Choisir l'exécutable installé", Filter = "Exécutable Windows|*.exe", CheckFileExists = true };
        if (picker.ShowDialog(this) == true) ExecutableBox.Text = picker.FileName;
    }
    private void OnBrowseCore(object sender, RoutedEventArgs e)
    {
        if (backend.Status.Busy || Installing) return;
        var picker = new OpenFileDialog { Title = "Choisir un cœur RetroArch adapté à la console", Filter = "Cœur Windows|*.dll", CheckFileExists = true };
        if (picker.ShowDialog(this) == true) CoreBox.Text = picker.FileName;
    }
    private void OnImport(object sender, RoutedEventArgs e) => Guard(() =>
    {
        if (backend.Status.Busy) throw new InvalidOperationException("Attendez la fin de la partie avant d'ajouter des jeux.");
        if (SelectedSystem is not { } system) return;
        if (system.Extensions.Length == 0) throw new InvalidOperationException("Aucun format ni module n'est encore intégré pour cette console.");
        var picker = new OpenFileDialog { Title = "Ajouter des jeux — " + system.Name,
            Filter = "Formats prévus|" + string.Join(';', system.Extensions.Select(extension => "*" + extension)), Multiselect = true, CheckFileExists = true };
        if (picker.ShowDialog(this) != true) return;
        backend.Import(system.Id, picker.FileNames);
        RefreshGames();
    });
    private void OnRemove(object sender, RoutedEventArgs e) => Guard(() =>
    {
        if (SelectedGame is not { } game) return;
        backend.Remove(game.Id);
        RefreshGames();
    });
    private Preparation? PrepareSelection()
    {
        if (SelectedGame is not { } game) return null;
        var preparation = backend.Prepare(game.Id);
        PreparationText.Text = preparation.Error ?? preparation.Summary;
        PlanText.Text = preparation.Request is { } request
            ? request.Executable + Environment.NewLine + string.Join(Environment.NewLine, request.Arguments.Select((argument, i) => $"Argument {i + 1} : {argument}")) : "";
        return preparation;
    }
    private void OnPrepare(object sender, RoutedEventArgs e) => PrepareSelection();
    private async void OnStart(object sender, RoutedEventArgs e)
    {
        if (backend.Status.Busy || Installing || SelectedGame is not { } game || PrepareSelection()?.CanLaunch != true) return;
        var previousState = WindowState;
        try
        {
            using var hotkey = new SessionHotkey(new WindowInteropHelper(this).Handle, () => backend.RequestStop());
            if (!hotkey.Registered)
                MessageBox.Show(this, "Le raccourci clavier est déjà utilisé ou indisponible. Utilisez Start + Back sur une manette XInput, le bouton de fermeture de l'atelier ou fermez le jeu normalement.",
                    "Retour au menu", MessageBoxButton.OK, MessageBoxImage.Information);
            sessionInput.Start();
            WindowState = WindowState.Minimized;
            await backend.StartAsync(game.Id);
        }
        catch (Exception ex) when (ex is InvalidOperationException or System.ComponentModel.Win32Exception)
        { StatusText.Text = ex.Message; }
        finally
        {
            sessionInput.Stop();
            WindowState = previousState;
            Activate();
            RefreshState();
        }
    }
    private void OnStop(object sender, RoutedEventArgs e) => backend.RequestStop();
    private async void OnInstallDolphin(object sender, RoutedEventArgs e)
    {
        if (backend.Status.Busy || Installing || !backend.CanWrite || SelectedSystem is not { } system) return;
        if (RuntimeInformation.OSArchitecture != Architecture.X64) return;
        installationCancellation = new();
        RefreshState();
        var progress = new Progress<InstallProgress>(p =>
        {
            DolphinInstallText.Text = p.Message;
            DolphinProgress.IsIndeterminate = p.Percent is null;
            if (p.Percent is { } percent) DolphinProgress.Value = percent;
        });
        string outcome;
        try
        {
            managedDolphin = await dolphinInstaller.InstallAsync(progress, installationCancellation.Token);
            backend.Configure(system.Id, new("dolphin", managedDolphin.Executable, UserDirectory: managedDolphin.UserDirectory));
            AdapterBox.SelectedItem = "dolphin";
            ExecutableBox.Text = managedDolphin.Executable;
            outcome = "Dolphin prêt pour " + system.Name + ". Ajoutez votre jeu dans Bibliothèque et lancement.";
        }
        catch (OperationCanceledException) when (installationCancellation.IsCancellationRequested)
        { outcome = "Installation annulée. Vous pouvez réessayer."; }
        catch (OperationCanceledException) { outcome = "Le téléchargement a dépassé le délai autorisé. Vérifiez la connexion puis réessayez."; }
        catch (Exception ex) { outcome = "Installation impossible : " + ex.Message; }
        finally
        {
            installationCancellation.Dispose();
            installationCancellation = null;
            DolphinProgress.IsIndeterminate = false;
            RefreshState();
        }
        DolphinInstallText.Text = outcome;
        StatusText.Text = outcome;
    }
    private void OnCancelInstall(object sender, RoutedEventArgs e) => installationCancellation?.Cancel();
    private void OnDolphinPrerequisite(object sender, RoutedEventArgs e) => Guard(() =>
        Process.Start(new ProcessStartInfo("https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist") { UseShellExecute = true }));
    private void OnExit(object sender, RoutedEventArgs e) => Close();
    private void OnOpenData(object sender, RoutedEventArgs e) => Guard(() =>
    {
        Directory.CreateDirectory(dataDirectory);
        Process.Start(new ProcessStartInfo("explorer.exe") { ArgumentList = { Path.GetFullPath(dataDirectory) }, UseShellExecute = false });
    });
    private void OnExport(object sender, RoutedEventArgs e) => Guard(() =>
    {
        var picker = new SaveFileDialog { Title = "Exporter le diagnostic local", Filter = "Texte|*.txt", FileName = "diagnostic-console-launcher.txt" };
        if (picker.ShowDialog(this) == true)
            File.WriteAllLines(picker.FileName, backend.Diagnostics.Select(entry => $"{entry.At:O} {entry.Message}"));
    });

    private async void OnLoaded(object sender, RoutedEventArgs e)
    {
        if (previewPath is null) return;
        var arguments = Environment.GetCommandLineArgs();
        var tabIndex = Array.IndexOf(arguments, "--preview-tab");
        if (tabIndex >= 0 && tabIndex + 1 < arguments.Length && int.TryParse(arguments[tabIndex + 1], out var tab))
            WorkspaceTabs.SelectedIndex = Math.Clamp(tab, 0, WorkspaceTabs.Items.Count - 1);
        await Dispatcher.InvokeAsync(() => { }, DispatcherPriority.ApplicationIdle);
        UpdateLayout();
        var image = new RenderTargetBitmap((int)ActualWidth, (int)ActualHeight, 96, 96, PixelFormats.Pbgra32);
        image.Render(this);
        var encoder = new PngBitmapEncoder();
        encoder.Frames.Add(BitmapFrame.Create(image));
        using (var output = File.Create(previewPath)) encoder.Save(output);
        Close();
    }
}
