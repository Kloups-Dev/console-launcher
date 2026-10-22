using System.Runtime.InteropServices;
using Launcher.Core;

namespace Launcher.Desktop;

internal sealed class Gamepad
{
    [StructLayout(LayoutKind.Sequential)] private struct Pad
    {
        public ushort Buttons;
        public byte LeftTrigger, RightTrigger;
        public short LeftX, LeftY, RightX, RightY;
    }
    [StructLayout(LayoutKind.Sequential)] private struct State { public uint Packet; public Pad Pad; }
    [DllImport("xinput1_4.dll")] private static extern uint XInputGetState(uint index, out State state);
    private ushort lastButtons;
    private NavigationAction? heldDirection;
    private long repeatAt;
    public bool Connected { get; private set; }

    public NavigationAction? Poll(bool acceptInput)
    {
        State state = default;
        Connected = false;
        for (uint i = 0; i < 4; i++)
        {
            if (XInputGetState(i, out state) != 0) continue;
            Connected = true;
            break;
        }
        var buttons = Connected ? state.Pad.Buttons : (ushort)0;
        var pressed = (ushort)(buttons & ~lastButtons);
        lastButtons = buttons;
        if (!acceptInput || !Connected) { heldDirection = null; return null; }
        if ((pressed & 0x1000) != 0) return NavigationAction.Confirm;
        if ((pressed & 0x2000) != 0) return NavigationAction.Back;
        NavigationAction? direction = null;
        if ((buttons & 4) != 0 || state.Pad.LeftX < -18000) direction = NavigationAction.Left;
        else if ((buttons & 8) != 0 || state.Pad.LeftX > 18000) direction = NavigationAction.Right;
        else if ((buttons & 1) != 0 || state.Pad.LeftY > 18000) direction = NavigationAction.Up;
        else if ((buttons & 2) != 0 || state.Pad.LeftY < -18000) direction = NavigationAction.Down;
        var now = Environment.TickCount64;
        if (direction != heldDirection)
        {
            heldDirection = direction;
            repeatAt = now + 350;
            return direction;
        }
        if (direction is not null && now >= repeatAt) { repeatAt = now + 140; return direction; }
        return null;
    }
}
