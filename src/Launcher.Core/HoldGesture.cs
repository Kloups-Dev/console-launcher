namespace Launcher.Core;

/// <summary>Triggers once per continuous hold, using a caller-supplied monotonic clock.</summary>
public sealed class HoldGesture(long durationMilliseconds)
{
    private long? startedAt;
    private bool fired;

    public bool Update(bool held, long now)
    {
        if (!held) { Reset(); return false; }
        if (startedAt is null || now < startedAt) { startedAt = now; return false; }
        if (fired || now - startedAt < durationMilliseconds) return false;
        fired = true;
        return true;
    }

    public void Reset() { startedAt = null; fired = false; }
}
