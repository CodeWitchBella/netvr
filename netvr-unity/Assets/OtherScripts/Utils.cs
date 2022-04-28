using UnityEngine;

static class Utils
{
    /**
     * Like Debug.Log but only appends stacktrace if in editor so that plaintext
     * logs are readable.
     */
    public static void Log(string text)
    {
#if UNITY_EDITOR
        Debug.Log(text);
#else
        Debug.LogFormat(LogType.Log, LogOption.NoStacktrace, null, text);
#endif
    }
}
