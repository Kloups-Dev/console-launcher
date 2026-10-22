namespace Launcher.Core;

public enum NavigationAction { Left, Right, Up, Down, Confirm, Back }
public enum LibraryView { Consoles, Games }

public sealed class NavigationState(int consoleCount)
{
    public int ConsoleIndex { get; private set; }
    public int GameIndex { get; private set; }
    public LibraryView View { get; private set; }

    // Returns true only when the user asks to launch a selected game.
    public bool Apply(NavigationAction action, int gameCount)
    {
        if (action == NavigationAction.Back) { View = LibraryView.Consoles; return false; }
        if (action == NavigationAction.Confirm)
        {
            if (View == LibraryView.Games) return gameCount > 0;
            View = LibraryView.Games;
            GameIndex = 0;
        }
        else if (View == LibraryView.Consoles && consoleCount > 0)
        {
            var delta = action == NavigationAction.Right ? 1 : action == NavigationAction.Left ? -1 : 0;
            ConsoleIndex = (ConsoleIndex + delta + consoleCount) % consoleCount;
            GameIndex = 0;
        }
        else if (View == LibraryView.Games && gameCount > 0)
        {
            var delta = action is NavigationAction.Right or NavigationAction.Down ? 1
                : action is NavigationAction.Left or NavigationAction.Up ? -1 : 0;
            GameIndex = (GameIndex + delta + gameCount) % gameCount;
        }
        return false;
    }

    public void SelectConsole(int index)
    {
        ConsoleIndex = Math.Clamp(index, 0, Math.Max(0, consoleCount - 1));
        GameIndex = 0;
        View = LibraryView.Consoles;
    }

    public void SelectGame(int index, int count)
    {
        GameIndex = Math.Clamp(index, 0, Math.Max(0, count - 1));
        View = LibraryView.Games;
    }
}
