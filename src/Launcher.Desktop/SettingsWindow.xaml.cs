using System.IO;
using System.Windows;
using Microsoft.Win32;

namespace Launcher.Desktop;

public partial class SettingsWindow : Window
{
    public string DolphinPath => ExecutablePath.Text.Trim();
    public SettingsWindow(string? dolphinPath)
    {
        InitializeComponent();
        ExecutablePath.Text = dolphinPath ?? "";
    }
    private void OnBrowse(object sender, RoutedEventArgs e)
    {
        var picker = new OpenFileDialog { Title = "Choisir Dolphin.exe", Filter = "Dolphin|Dolphin.exe", CheckFileExists = true };
        if (picker.ShowDialog(this) == true) ExecutablePath.Text = picker.FileName;
    }
    private void OnSave(object sender, RoutedEventArgs e)
    {
        if (!File.Exists(DolphinPath) || !string.Equals(Path.GetFileName(DolphinPath), "Dolphin.exe", StringComparison.OrdinalIgnoreCase))
        { ErrorLabel.Text = "Choisissez un fichier Dolphin.exe existant."; return; }
        DialogResult = true;
    }
}
