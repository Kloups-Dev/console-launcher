using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Threading;
using Launcher.Core;
using Microsoft.Win32;

namespace Launcher.Desktop;

public partial class MainWindow : Window
{
    private readonly NavigationState navigation = new(Catalog.Systems.Count);
    private readonly SessionRunner sessions = new();
    private readonly Gamepad gamepad = new();
    private readonly DispatcherTimer inputTimer = new() { Interval = TimeSpan.FromMilliseconds(25) };
    private readonly LibraryStore store;
    private LibraryData library = new();
    private bool rendering, modal, storageBlocked;
    private string? loadError;
    private IReadOnlyList<GameEntry> games = [];

    public MainWindow()
    {
        InitializeComponent();
        var arguments = Environment.GetCommandLineArgs();
        var dataIndex = Array.IndexOf(arguments, "--data-dir");
        var directory = dataIndex >= 0 && dataIndex + 1 < arguments.Length ? arguments[dataIndex + 1]
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "ConsoleLauncher");
        store = new(Path.Combine(directory, "library.json"));
        try { library = store.Load(); }
        catch (Exception ex) when (ex is IOException or InvalidDataException or System.Text.Json.JsonException or UnauthorizedAccessException)
        { storageBlocked = true; loadError = "Bibliothèque illisible : le fichier est conservé. " + ex.Message; }
        foreach (var system in Catalog.Systems)
        {
            var button = new Button { Content = system.Name, Tag = system.Id, Margin = new Thickness(0, 0, 12, 0), Padding = new Thickness(22, 16, 22, 16), MinWidth = 190 };
            var index = ConsoleStrip.Children.Count;
            button.Click += (_, _) => { navigation.SelectConsole(index); Render(); };
            ConsoleStrip.Children.Add(button);
        }
        inputTimer.Tick += (_, _) =>
        {
            var action = gamepad.Poll(IsActive && !modal && !sessions.IsRunning);
            ControllerLabel.Text = gamepad.Connected ? "Manette Xbox compatible connectée" : "Clavier · Manette XInput non détectée";
            if (action.HasValue) Navigate(action.Value);
        };
        inputTimer.Start();
        Render();
        if (loadError is not null) StatusLabel.Text = loadError;
        Closing += (_, e) => { if (sessions.IsRunning) { e.Cancel = true; StatusLabel.Text = "Fermez la partie avant de quitter le lanceur."; } };
        Closed += (_, _) => inputTimer.Stop();
        Loaded += OnLoaded;
    }

    private void Render()
    {
        rendering = true;
        try
        {
            var system = Catalog.Systems[navigation.ConsoleIndex];
            var realGames = library.Games.Where(g => g.SystemId == system.Id).ToList();
            games = realGames.Count > 0 ? realGames : Catalog.DemoGames.Where(g => g.SystemId == system.Id).ToList();
            SystemTitle.Text = system.Name;
            SystemDescription.Text = system.Description;
            SystemMaker.Text = system.Maker;
            CountLabel.Text = realGames.Count > 0 ? $"{realGames.Count} jeu(x) ajouté(s)" : "Aperçu · Titres fictifs";
            Eyebrow.Text = navigation.View == LibraryView.Consoles ? "01 / VOS CONSOLES" : "02 / CHOISIR UN JEU";
            foreach (Button button in ConsoleStrip.Children)
            {
                var selected = (string)button.Tag == system.Id;
                button.Background = Brush(selected ? "#274236" : "#192232");
                button.BorderBrush = Brush(selected ? "#92E5C5" : "#354258");
            }
            GameList.ItemsSource = games;
            GameList.SelectedIndex = games.Count > 0 ? Math.Min(navigation.GameIndex, games.Count - 1) : -1;
            if (GameList.SelectedItem is not null) GameList.ScrollIntoView(GameList.SelectedItem);
            var game = GameList.SelectedItem as GameEntry;
            GameMark.Text = system.Mark;
            GameTitle.Text = game?.Title ?? "Votre collection commence ici";
            GameDetail.Text = game?.FilePath is null ? "Titre fictif pour explorer le menu. Ajoutez votre propre jeu pour lancer une partie."
                : "Votre fichier est prêt. Dolphin utilisera ses réglages et sauvegardes existants.";
            PlayButton.Content = game?.FilePath is null ? "Aperçu uniquement" : "Jouer";
            PlayButton.IsEnabled = game?.FilePath is not null && system.Id == "gamecube" && !sessions.IsRunning;
            ImportButton.IsEnabled = system.Id == "gamecube" && !storageBlocked && !sessions.IsRunning;
        }
        finally { rendering = false; }
    }

    private static SolidColorBrush Brush(string color) => new((Color)ColorConverter.ConvertFromString(color));

    private async void Navigate(NavigationAction action)
    {
        if (sessions.IsRunning || modal) return;
        var launch = navigation.Apply(action, games.Count);
        Render();
        if (launch) await LaunchSelectedAsync();
    }

    private void OnKeyDown(object sender, KeyEventArgs e)
    {
        if (e.Key == Key.F11) { ToggleFullscreen(); e.Handled = true; return; }
        // Preserve native keyboard activation of settings/import/play buttons.
        if (e.Key == Key.Enter && Keyboard.FocusedElement is Button focused &&
            (focused == SettingsButton || focused == ImportButton || focused == PlayButton)) return;
        NavigationAction? action = e.Key switch
        {
            Key.Left => NavigationAction.Left, Key.Right => NavigationAction.Right,
            Key.Up => NavigationAction.Up, Key.Down => NavigationAction.Down,
            Key.Enter => NavigationAction.Confirm, Key.Escape => NavigationAction.Back, _ => null
        };
        if (action.HasValue) { e.Handled = true; if (!e.IsRepeat || action is not NavigationAction.Confirm) Navigate(action.Value); }
    }

    private void ToggleFullscreen()
    {
        if (WindowStyle == WindowStyle.None) { WindowStyle = WindowStyle.SingleBorderWindow; WindowState = WindowState.Normal; }
        else { WindowStyle = WindowStyle.None; WindowState = WindowState.Maximized; }
    }

    private void OnGameSelected(object sender, SelectionChangedEventArgs e)
    {
        if (rendering || GameList.SelectedIndex < 0) return;
        navigation.SelectGame(GameList.SelectedIndex, games.Count);
        Render();
    }

    private async void OnGameDoubleClick(object sender, MouseButtonEventArgs e) => await LaunchSelectedAsync();
    private async void OnPlay(object sender, RoutedEventArgs e) => await LaunchSelectedAsync();

    private bool ShowSettings()
    {
        modal = true;
        try
        {
            var window = new SettingsWindow(library.DolphinPath) { Owner = this };
            if (window.ShowDialog() != true) return false;
            if (storageBlocked) throw new IOException("La bibliothèque doit être réparée avant d'enregistrer de nouveaux réglages.");
            var previous = library.DolphinPath;
            library.DolphinPath = window.DolphinPath;
            try { store.Save(library); }
            catch { library.DolphinPath = previous; throw; }
            StatusLabel.Text = "Dolphin sélectionné. Vous pouvez ajouter votre jeu GameCube.";
            return true;
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        { StatusLabel.Text = ex.Message; return false; }
        finally { modal = false; }
    }

    private void OnSettings(object sender, RoutedEventArgs e) => ShowSettings();

    private void OnImport(object sender, RoutedEventArgs e)
    {
        modal = true;
        try
        {
            var picker = new OpenFileDialog { Title = "Ajouter un jeu GameCube", Filter = "Jeux GameCube|*.iso;*.gcm;*.rvz;*.gcz;*.ciso", CheckFileExists = true };
            if (picker.ShowDialog(this) != true) return;
            var before = library.Games.ToList();
            var game = LibraryStore.ImportGameCube(library, picker.FileName);
            try { store.Save(library); }
            catch { library.Games = before; throw; }
            navigation.SelectConsole(0);
            navigation.SelectGame(library.Games.Where(g => g.SystemId == "gamecube").ToList().FindIndex(g => g.Id == game.Id), library.Games.Count);
            Render();
            StatusLabel.Text = "Jeu ajouté. Le format est accepté ; son contenu sera vérifié par Dolphin au lancement.";
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        { StatusLabel.Text = "Ajout impossible : " + ex.Message; }
        finally { modal = false; }
    }

    private async Task LaunchSelectedAsync()
    {
        if (sessions.IsRunning || modal) return;
        if (GameList.SelectedItem is not GameEntry game || game.FilePath is null)
        { StatusLabel.Text = "Ceci est un titre fictif. Ajoutez votre fichier de jeu GameCube."; return; }
        if (string.IsNullOrWhiteSpace(library.DolphinPath) && !ShowSettings()) return;
        try
        {
            var request = new DolphinAdapter().Prepare(game, library.DolphinPath!);
            inputTimer.Stop();
            IsEnabled = false;
            StatusLabel.Text = "Partie en cours. Fermez le jeu dans Dolphin pour revenir ici.";
            // Hide only after validation. The finally block restores the launcher on start failure too.
            Hide();
            var exit = await sessions.RunAsync(request);
            StatusLabel.Text = exit == 0 ? "De retour dans votre bibliothèque." : $"La partie s'est arrêtée (code {exit}). Vérifiez les réglages Dolphin.";
        }
        catch (Exception ex) when (ex is IOException or InvalidOperationException or System.ComponentModel.Win32Exception)
        { StatusLabel.Text = "Lancement impossible : " + ex.Message; }
        finally
        {
            IsEnabled = true;
            Show();
            Activate();
            inputTimer.Start();
            Render();
        }
    }

    private async void OnLoaded(object sender, RoutedEventArgs e)
    {
        // Deterministic offscreen rendering of our own UI for layout QA; no desktop capture.
        var arguments = Environment.GetCommandLineArgs();
        var index = Array.IndexOf(arguments, "--preview");
        if (index < 0 || index + 1 >= arguments.Length) return;
        await Dispatcher.InvokeAsync(() => { }, DispatcherPriority.ApplicationIdle);
        UpdateLayout();
        var bitmap = new RenderTargetBitmap((int)ActualWidth, (int)ActualHeight, 96, 96, PixelFormats.Pbgra32);
        bitmap.Render(this);
        var encoder = new PngBitmapEncoder();
        encoder.Frames.Add(BitmapFrame.Create(bitmap));
        using (var output = File.Create(arguments[index + 1])) encoder.Save(output);
        Close();
    }
}
