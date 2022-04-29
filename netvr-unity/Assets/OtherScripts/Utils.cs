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

    /**
     * Like Debug.Log but only appends stacktrace if in editor so that plaintext
     * logs are readable.
     */
    public static void Log(string text)
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

        text = text.Replace("\n", "\n    ");
#if UNITY_EDITOR
        Debug.Log(text);
#else
        Debug.LogFormat(LogType.Log, LogOption.NoStacktrace, null, "{0}", text);
#endif
    }
}
