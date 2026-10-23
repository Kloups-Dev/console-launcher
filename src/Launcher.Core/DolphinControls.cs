namespace Launcher.Core;

public enum DolphinInputMode { Keyboard, Gamepad, Both }

public static class DolphinControls
{
    public const string KeyboardHelp = "Stick principal : flèches • Stick C : I / K / J / L\n"
        + "Boutons GameCube : A = X, B = Z, X = C, Y = S, Z = D\n"
        + "Start = Entrée • L = Q, R = W • Croix : T / G / F / H (haut / bas / gauche / droite)";

    public static string CreateProfile(DolphinInputMode mode)
    {
        if (!Enum.IsDefined(mode)) throw new ArgumentOutOfRangeException(nameof(mode));
        string Bind(string keyboard, string gamepad)
        {
            var key = "`DInput/0/Keyboard Mouse:" + keyboard + "`";
            var pad = "`XInput/0/Gamepad:" + gamepad + "`";
            return mode switch { DolphinInputMode.Keyboard => key, DolphinInputMode.Gamepad => pad, _ => key + " | " + pad };
        }
        return string.Join('\n', new[]
        {
            "[GCPad1]", "; ConsoleLauncher input: " + mode,
            "Device = " + (mode == DolphinInputMode.Gamepad ? "XInput/0/Gamepad" : "DInput/0/Keyboard Mouse"),
            "Buttons/A = " + Bind("X", "Button A"), "Buttons/B = " + Bind("Z", "Button B"),
            "Buttons/X = " + Bind("C", "Button X"), "Buttons/Y = " + Bind("S", "Button Y"),
            "Buttons/Z = " + Bind("D", "Shoulder R"), "Buttons/Start = " + Bind("RETURN", "Start"),
            "Main Stick/Up = " + Bind("UP", "Left Y+"), "Main Stick/Down = " + Bind("DOWN", "Left Y-"),
            "Main Stick/Left = " + Bind("LEFT", "Left X-"), "Main Stick/Right = " + Bind("RIGHT", "Left X+"),
            "Main Stick/Dead Zone = 15", "C-Stick/Dead Zone = 15",
            "C-Stick/Up = " + Bind("I", "Right Y+"), "C-Stick/Down = " + Bind("K", "Right Y-"),
            "C-Stick/Left = " + Bind("J", "Right X-"), "C-Stick/Right = " + Bind("L", "Right X+"),
            "Triggers/L = " + Bind("Q", "Trigger L"), "Triggers/R = " + Bind("W", "Trigger R"),
            "Triggers/L-Analog = " + Bind("Q", "Trigger L"), "Triggers/R-Analog = " + Bind("W", "Trigger R"),
            "D-Pad/Up = " + Bind("T", "Pad N"), "D-Pad/Down = " + Bind("G", "Pad S"),
            "D-Pad/Left = " + Bind("F", "Pad W"), "D-Pad/Right = " + Bind("H", "Pad E"), ""
        });
    }

    // Explicit user action: replace only player 1 and keep an exact backup of the original file.
    public static string? Apply(string userDirectory, DolphinInputMode mode)
    {
        var profile = CreateProfile(mode);
        var config = Path.Combine(userDirectory, "Config");
        Directory.CreateDirectory(config);
        var path = Path.Combine(config, "GCPadNew.ini");
        var original = File.Exists(path) ? File.ReadAllText(path) : "";
        var output = new List<string>();
        var insidePlayer = false;
        var replaced = false;
        foreach (var line in original.Replace("\r\n", "\n").Split('\n'))
        {
            var trimmed = line.Trim();
            if (trimmed.StartsWith('[') && trimmed.Contains(']'))
            {
                var section = trimmed[..(trimmed.IndexOf(']') + 1)];
                insidePlayer = section.Equals("[GCPad1]", StringComparison.OrdinalIgnoreCase);
                if (insidePlayer)
                {
                    if (!replaced) output.Add(profile);
                    replaced = true;
                    continue;
                }
            }
            if (!insidePlayer) output.Add(line);
        }
        if (!replaced) output.Add(profile);
        var next = string.Join('\n', output);
        if (original.Replace("\r\n", "\n") == next) return null;
        var backup = File.Exists(path) ? path + ".backup-" + Guid.NewGuid().ToString("N") : null;
        if (backup is not null) File.Copy(path, backup);
        var temporary = path + "." + Guid.NewGuid().ToString("N") + ".tmp";
        try { File.WriteAllText(temporary, next); File.Move(temporary, path, true); }
        finally { if (File.Exists(temporary)) File.Delete(temporary); }
        return backup;
    }
}
