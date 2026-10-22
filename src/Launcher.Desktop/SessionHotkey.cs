using System.Runtime.InteropServices;
using System.Windows.Interop;

namespace Launcher.Desktop;

internal sealed class SessionHotkey : IDisposable
{
    private const int Id = 0x4C43;
    private readonly HwndSource source;
    private readonly Action onReturn;
    public bool Registered { get; }

    [DllImport("user32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool RegisterHotKey(IntPtr window, int id, uint modifiers, uint key);
    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool UnregisterHotKey(IntPtr window, int id);

    public SessionHotkey(IntPtr handle, Action onReturn)
    {
        this.onReturn = onReturn;
        source = HwndSource.FromHwnd(handle) ?? throw new InvalidOperationException("La fenêtre du lanceur n'est pas prête.");
        source.AddHook(OnMessage);
        // Ctrl + Alt + Backspace, no key-repeat. Reserved only while a game session is active.
        Registered = RegisterHotKey(handle, Id, 0x4003, 0x08);
    }

    private IntPtr OnMessage(IntPtr hwnd, int message, IntPtr wParam, IntPtr lParam, ref bool handled)
    {
        if (Registered && message == 0x0312 && wParam.ToInt32() == Id)
        {
            onReturn();
            handled = true;
        }
        return IntPtr.Zero;
    }

    public void Dispose()
    {
        if (Registered) UnregisterHotKey(source.Handle, Id);
        source.RemoveHook(OnMessage);
    }
}
