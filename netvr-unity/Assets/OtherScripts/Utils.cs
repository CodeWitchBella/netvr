using System;
using System.Collections.Generic;
using UnityEngine;

static class Utils
{
    const int Length = 1024;
    static readonly string[] _log = new string[Length];
    static int _index = 0;
    static bool _wrapped = false;

    public static string ReadLog()
    {
        string res = "";
        if (_wrapped)
        {
            for (int i = 0; i < Length; ++i)
                res += _log[(i + _index) % Length] + "\n";
        }
        else
        {
            for (int i = 0; i < _index; ++i)
                res += _log[i] + "\n";
        }
        return res;
    }

    private static void AppendLines(string text)
    {
        var lines = text.Split("\n");
        for (int i = 0; i < lines.Length; ++i)
        {
            _log[_index] = lines[i];
            ++_index;
            if (_index >= Length)
            {
                _index = 0;
                _wrapped = false;
            }
        }
    }

    /**
     * Like Debug.Log but only appends stacktrace if in editor so that plaintext
     * logs are readable. Also appends to senable log.
     */
    public static void Log(string text)
    {
        text = text.Replace("\n", "\n    ");
        AppendLines(text);

#if UNITY_EDITOR
        Debug.LogFormat(LogType.Log, LogOption.None, null, "{0}", text);
#else
        Debug.LogFormat(LogType.Log, LogOption.NoStacktrace, null, "{0}", text);
#endif
    }

    /**
     * Like Debug.LogException but also appends to sendable log.
     */
    public static void LogException(Exception error)
    {
        var text = error.Message + "\n" + error.StackTrace;
        text = text.Replace("\n", "\n    ");
        AppendLines(text);

#if UNITY_EDITOR
        Debug.LogException(error);
#else
        Debug.LogFormat(LogType.Exception, LogOption.NoStacktrace, null, "{0}", text);
#endif
    }


    /**
    * Like Debug.LogWarning but only appends stacktrace if in editor so that
    * plaintext logs are readable. Also appends to senable log.
    */
    public static void LogWarning(string text)
    {
        text = text.Replace("\n", "\n    ");
        AppendLines(text);

#if UNITY_EDITOR
        Debug.LogWarning(text);
#else
        Debug.LogFormat(LogType.Exception, LogOption.NoStacktrace, null, "{0}", text);
#endif
    }
}
