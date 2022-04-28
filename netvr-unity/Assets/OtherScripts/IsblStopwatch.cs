using System;
using System.Diagnostics;

class IsblStopwatch : IDisposable
{
    readonly Stopwatch _stopWatch;
    readonly string _label;
    public IsblStopwatch(string label)
    {
        _label = label;
        _stopWatch = new();
        _stopWatch.Start();
    }

    public void Dispose()
    {
        _stopWatch.Stop();
        // Get the elapsed time as a TimeSpan value.
        TimeSpan ts = _stopWatch.Elapsed;

        // Format and display the TimeSpan value.
        string elapsedTime = String.Format("{0:00}:{1:00}:{2:00}.{3:000}",
            ts.Hours, ts.Minutes, ts.Seconds, ts.Milliseconds);
        UnityEngine.Debug.Log($"IsblStopWatch({_label}): {elapsedTime}");
    }
}
