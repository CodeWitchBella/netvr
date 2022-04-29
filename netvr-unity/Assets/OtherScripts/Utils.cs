using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;
using UnityEngine;

static class Utils
{
    struct LogEntry
    {
        [JsonInclude]
        [JsonPropertyName("text")]
        public string Text;

        [JsonInclude]
        [JsonPropertyName("type")]
        [JsonConverter(typeof(Isbl.Json.EnumToStringConverter))]
        public LogType Type;
    }

    const int Length = 1024;
    static readonly LogEntry[] _log = new LogEntry[Length];
    static int _index = 0;
    static bool _wrapped = false;

    public static object ReadLog()
    {
        if (!_wrapped)
        {
            LogEntry[] res = new LogEntry[_index];
            for (int i = 0; i < _index; ++i)
                res[i] = _log[i];
            return res;
        }
        else
        {
            LogEntry[] res = new LogEntry[Length];
            for (int i = 0; i < Length; ++i)
                res[i] = _log[(i + _index) % Length];
            return res;
        }
    }

    private static void AppendEntry(LogEntry entry)
    {
        _log[_index] = entry;
        ++_index;
        if (_index >= Length)
        {
            _index = 0;
            _wrapped = false;
        }
    }

    /**
     * Like Debug.Log but only appends stacktrace if in editor so that plaintext
     * logs are readable. Also appends to senable log.
     */
    public static void Log(string text)
    {
        text = text.Replace("\n", "\n    ");
        AppendEntry(new LogEntry() { Text = text, Type = LogType.Log });

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
        AppendEntry(new LogEntry() { Text = text, Type = LogType.Exception });

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
        AppendEntry(new LogEntry() { Text = text, Type = LogType.Warning });

#if UNITY_EDITOR
        Debug.LogWarning(text);
#else
        Debug.LogFormat(LogType.Warning, LogOption.NoStacktrace, null, "{0}", text);
#endif
    }
}
