using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Input;
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
    private readonly HashSet<DiagnosticEntry> loggedBackend = [];
    private readonly InputProbe inputProbe = new();
    private readonly DispatcherTimer inputTestTimer = new() { Interval = TimeSpan.FromMilliseconds(250) };
    private long inputTestDeadline;
    private bool? lastPadConnected;
    private bool diagnosticReady;
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
        backend = new LauncherBackend(new LibraryStore(Path.Combine(dataDirectory, "library.json")), new SessionRunner(App.Diagnostics));
        dolphinInstaller = new(downloadClient, dataDirectory);
        managedDolphin = dolphinInstaller.Detect();
        KeyboardHelpText.Text = DolphinControls.KeyboardHelp;
        RemoteInputBox.IsChecked = App.RemoteDeclared;
        diagnosticReady = true;
        UpdateDiagnosticContext();
        RecordBackendDiagnostics();
        AddHandler(ButtonBase.ClickEvent, new RoutedEventHandler((_, e) =>
        {
            if (e.Source is ButtonBase { Content: string label }) App.Diagnostics.Write("ui.action", label);
        }), true);
        Activated += (_, _) => App.Diagnostics.Write("ui.focus", "Atelier actif.", DiagnosticLevel.Observation);
        Deactivated += (_, _) => App.Diagnostics.Write("ui.focus", "Atelier hors focus.", DiagnosticLevel.Observation);
        inputTestTimer.Tick += (_, _) =>
        {
            InputTestText.Text = inputProbe.Summary(App.RemoteContext);
            if (Environment.TickCount64 >= inputTestDeadline) FinishInputTest();
        };
        backend.Changed += OnBackendChanged;
        SystemsList.ItemsSource = IntegrationCatalog.Systems;
        SystemsList.SelectedIndex = 0;
        sessionInput.Tick += (_, _) =>
        {
            gamepad.Poll(false, backend.Status.Busy);
            if (lastPadConnected != gamepad.Connected)
            {
                lastPadConnected = gamepad.Connected;
                App.Diagnostics.Write("input.xinput", "Manette XInput détectée=" + gamepad.Connected + "; contexte distant=" + App.RemoteContext, DiagnosticLevel.Observation);
            }
            if (gamepad.ReturnRequested) backend.RequestStop();
        };
        Closing += (_, e) => { if (backend.Status.Busy || Installing) { e.Cancel = true; StatusText.Text = "Terminez la partie ou annulez l'installation avant de quitter l'atelier."; } };
        Closed += (_, _) => { FinishInputTest(); sessionInput.Stop(); backend.Changed -= OnBackendChanged; };
        Loaded += OnLoaded;
        RefreshState();
    }

    private void OnBackendChanged(object? sender, EventArgs e)
    {
        RecordBackendDiagnostics();
        if (Dispatcher.CheckAccess()) RefreshState();
        else Dispatcher.BeginInvoke(new Action(RefreshState));
    }

    private void RecordBackendDiagnostics()
    {
        lock (loggedBackend)
        {
            var entries = backend.Diagnostics;
            foreach (var entry in entries)
                if (loggedBackend.Add(entry)) App.Diagnostics.Write("backend", entry.Message,
                    backend.Status.Phase == SessionPhase.Failed ? DiagnosticLevel.Error : DiagnosticLevel.Info);
            loggedBackend.IntersectWith(entries);
        }
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
        DolphinControlsPanel.Visibility = system?.Id == "gamecube" ? Visibility.Visible : Visibility.Collapsed;
        ApplyDolphinControlsButton.IsEnabled = !busy && backend.CanWrite && managedDolphin is not null;
        DolphinInputBox.IsEnabled = !busy;
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
            ControlsResultText.Text = "Les réglages existants sont conservés tant que vous ne cliquez pas sur Appliquer les contrôles.";
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
        { App.Diagnostics.Write("ui.error", ex.ToString(), DiagnosticLevel.Error); StatusText.Text = ex.Message; MessageBox.Show(this, ex.Message, "Action impossible", MessageBoxButton.OK, MessageBoxImage.Information); }
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
        App.Diagnostics.Write("launch.prepare", preparation.Error ?? preparation.Summary,
            preparation.CanLaunch ? DiagnosticLevel.Info : DiagnosticLevel.Error);
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
            App.Diagnostics.Write("input.return-shortcut", "Raccourci retour enregistré=" + hotkey.Registered, DiagnosticLevel.Observation);
            if (!hotkey.Registered)
                MessageBox.Show(this, "Le raccourci clavier est déjà utilisé ou indisponible. Utilisez Start + Back sur une manette XInput, le bouton de fermeture de l'atelier ou fermez le jeu normalement.",
                    "Retour au menu", MessageBoxButton.OK, MessageBoxImage.Information);
            sessionInput.Start();
            WindowState = WindowState.Minimized;
            await backend.StartAsync(game.Id);
        }
        catch (Exception ex) when (ex is InvalidOperationException or System.ComponentModel.Win32Exception)
        { App.Diagnostics.Write("launch.error", ex.ToString(), DiagnosticLevel.Error); StatusText.Text = ex.Message; }
        finally
        {
            sessionInput.Stop();
            WindowState = previousState;
            Activate();
            RefreshState();
            CollectDolphinLog(game.SystemId);
        }
    }
    private void OnStop(object sender, RoutedEventArgs e) => backend.RequestStop();
    private async void OnInstallDolphin(object sender, RoutedEventArgs e)
    {
        if (backend.Status.Busy || Installing || !backend.CanWrite || SelectedSystem is not { } system) return;
        if (RuntimeInformation.OSArchitecture != Architecture.X64) return;
        installationCancellation = new();
        App.Diagnostics.Write("install.start", "Dolphin " + DolphinPackage.Current.Version);
        RefreshState();
        var progress = new Progress<InstallProgress>(p =>
        {
            DolphinInstallText.Text = p.Message;
            DolphinProgress.IsIndeterminate = p.Percent is null;
            if (p.Percent is { } percent) DolphinProgress.Value = percent;
        });
        string outcome;
        var outcomeLevel = DiagnosticLevel.Info;
        try
        {
            managedDolphin = await dolphinInstaller.InstallAsync(progress, installationCancellation.Token);
            backend.Configure(system.Id, new("dolphin", managedDolphin.Executable, UserDirectory: managedDolphin.UserDirectory));
            AdapterBox.SelectedItem = "dolphin";
            ExecutableBox.Text = managedDolphin.Executable;
            outcome = "Dolphin prêt pour " + system.Name + ". Ajoutez votre jeu dans Bibliothèque et lancement.";
        }
        catch (OperationCanceledException) when (installationCancellation.IsCancellationRequested)
        { outcome = "Installation annulée. Vous pouvez réessayer."; outcomeLevel = DiagnosticLevel.Observation; }
        catch (OperationCanceledException) { outcome = "Le téléchargement a dépassé le délai autorisé. Vérifiez la connexion puis réessayez."; outcomeLevel = DiagnosticLevel.Error; }
        catch (Exception ex) { outcome = "Installation impossible : " + ex.Message; outcomeLevel = DiagnosticLevel.Error; App.Diagnostics.Write("install.error", ex.ToString(), DiagnosticLevel.Error); }
        finally
        {
            installationCancellation.Dispose();
            installationCancellation = null;
            DolphinProgress.IsIndeterminate = false;
            RefreshState();
        }
        DolphinInstallText.Text = outcome;
        StatusText.Text = outcome;
        App.Diagnostics.Write("install.result", outcome, outcomeLevel);
    }
    private void OnCancelInstall(object sender, RoutedEventArgs e) => installationCancellation?.Cancel();
    private void OnApplyDolphinControls(object sender, RoutedEventArgs e) => Guard(() =>
    {
        if (backend.Status.Busy) throw new InvalidOperationException("Fermez la partie avant de modifier les contrôles.");
        if (managedDolphin is null || SelectedSystem?.Id != "gamecube")
            throw new InvalidOperationException("Installez ou sélectionnez d'abord Dolphin depuis l'atelier.");
        var tag = (DolphinInputBox.SelectedItem as ComboBoxItem)?.Tag as string;
        if (!Enum.TryParse<DolphinInputMode>(tag, out var mode)) return;
        backend.Configure("gamecube", new("dolphin", managedDolphin.Executable, UserDirectory: managedDolphin.UserDirectory));
        var backup = DolphinControls.Apply(managedDolphin.UserDirectory, mode);
        App.Diagnostics.Write("input.profile", "Profil GameCube=" + mode + "; sauvegarde précédente=" + (backup is not null));
        AdapterBox.SelectedItem = "dolphin";
        ExecutableBox.Text = managedDolphin.Executable;
        ControlsResultText.Text = backup is null ? "Contrôles appliqués. Prêts pour le prochain lancement."
            : "Contrôles appliqués. Copie des anciens réglages : " + backup;
    });
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
        {
            App.Diagnostics.Write("diagnostic.export", "Export demandé par l'utilisateur.");
            File.WriteAllLines(picker.FileName, App.Diagnostics.Snapshot());
        }
    });

    private void OnOpenLogs(object sender, RoutedEventArgs e) => Guard(() =>
    {
        Directory.CreateDirectory(App.Diagnostics.DirectoryPath);
        Process.Start(new ProcessStartInfo("explorer.exe") { ArgumentList = { App.Diagnostics.DirectoryPath }, UseShellExecute = false });
    });
    private void UpdateDiagnosticContext()
    {
        RemoteContextText.Text = $"Session distante Windows : {(App.RemoteDetected ? "détectée" : "non détectée")}. "
            + "Une application de streaming peut ne pas être détectée : cochez le mode distant. Aucun écart d'entrée n'est classé automatiquement comme un bug.";
        LogLocationText.Text = "Journaux locaux : " + App.Diagnostics.DirectoryPath + " · 5 fichiers limités · aucun envoi automatique."
            + (App.Diagnostics.LastWriteError is { } error ? " Écriture indisponible : " + error : "");
    }
    private void OnRemoteInputChanged(object sender, RoutedEventArgs e)
    {
        if (!diagnosticReady) return;
        App.SetRemoteDeclared(RemoteInputBox.IsChecked == true);
        UpdateDiagnosticContext();
    }
    private void OnStartInputTest(object sender, RoutedEventArgs e)
    {
        inputProbe.Start();
        inputTestDeadline = Environment.TickCount64 + 10000;
        InputTestSurface.Focus();
        StartInputTestButton.IsEnabled = false;
        StopInputTestButton.IsEnabled = true;
        inputTestTimer.Start();
        App.Diagnostics.Write("input.test.start", "Test dans la zone de l'atelier uniquement. Contexte distant=" + App.RemoteContext, DiagnosticLevel.Observation);
    }
    private void OnStopInputTest(object sender, RoutedEventArgs e) => FinishInputTest();
    private void FinishInputTest()
    {
        if (!inputProbe.Active) return;
        var result = inputProbe.Summary(App.RemoteContext);
        inputProbe.Stop();
        inputTestTimer.Stop();
        InputTestText.Text = result;
        StartInputTestButton.IsEnabled = true;
        StopInputTestButton.IsEnabled = false;
        App.Diagnostics.Write("input.test.result", result, DiagnosticLevel.Observation);
    }
    private void OnProbeKeyDown(object sender, KeyEventArgs e) => inputProbe.Key(true, e.IsRepeat);
    private void OnProbeKeyUp(object sender, KeyEventArgs e) => inputProbe.Key(false);
    private void OnProbeMouseMove(object sender, MouseEventArgs e) => inputProbe.Mouse(false);
    private void OnProbeMouseDown(object sender, MouseButtonEventArgs e) { InputTestSurface.Focus(); inputProbe.Mouse(true); }
    private void OnProbeFocusLost(object sender, KeyboardFocusChangedEventArgs e) => inputProbe.LoseFocus();
    private void CollectDolphinLog(string systemId)
    {
        if (backend.Configuration(systemId) is not { AdapterId: "dolphin", UserDirectory: { } user }) return;
        var path = Path.Combine(user, "Logs", "dolphin.log");
        try
        {
            if (!File.Exists(path)) { App.Diagnostics.Write("dolphin.log", "Aucun journal interne Dolphin disponible. Sorties standard collectées si émises.", DiagnosticLevel.Observation); return; }
            App.Diagnostics.Write("dolphin.log", "Extrait du journal interne existant, dernière écriture=" + File.GetLastWriteTimeUtc(path).ToString("O") + ". Peut inclure une session précédente.", DiagnosticLevel.Observation);
            using var stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite);
            stream.Seek(Math.Max(0, stream.Length - 16384), SeekOrigin.Begin);
            using var reader = new StreamReader(stream);
            var tail = reader.ReadToEnd();
            for (var offset = 0; offset < tail.Length; offset += 4000)
                App.Diagnostics.Write("dolphin.log.tail", tail.Substring(offset, Math.Min(4000, tail.Length - offset)), DiagnosticLevel.Observation);
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        { App.Diagnostics.Write("dolphin.log", ex.Message, DiagnosticLevel.Observation); }
    }

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
