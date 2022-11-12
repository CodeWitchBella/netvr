using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;
using UnityEngine;

namespace Isbl.NetVR
{

    static class Utils
    {


        /**
         * Like Debug.Log but only appends stacktrace if in editor so that plaintext
         * logs are readable. Also appends to senable log.
         */
        public static void Log(string text)
        {
            text = text.Replace("\n", "\n    ");

#if UNITY_EDITOR
            Debug.LogFormat(LogType.Log, LogOption.None, null, "{0}", text);
#else
            Debug.LogFormat(LogType.Log, LogOption.NoStacktrace, null, "{0}", text);
#endif
        }

        /**
         * Logs json in such a way that it could be syntax haighlighted in the GUI
         */
        public static void LogJson(string label, string json)
        {

#if UNITY_EDITOR
            Debug.LogFormat(LogType.Log, LogOption.None, null, "{0} {1}", label, json);
#else
            Debug.LogFormat(LogType.Log, LogOption.NoStacktrace, null, "{0} {1}", label, json);
#endif
        }

        /**
        * Logs json in such a way that it could be syntax haighlighted in the GUI
        */
        public static void LogJson(string label, object obj)
        {
            string json = System.Text.Json.JsonSerializer.Serialize(obj);
            LogJson(label, json);
        }

        /**
         * Like Debug.LogException but also appends to sendable log.
         */
        public static void LogException(Exception error)
        {
            var text = error.Message + "\n" + error.StackTrace;
            text = text.Replace("\n", "\n    ");

#if UNITY_EDITOR
            Debug.LogException(error);
#else
            Debug.LogFormat(LogType.Exception, LogOption.NoStacktrace, null, "{0}", text);
#endif
        }


        /**
        * Like Debug.LogWarning but only appends stacktrace if in editor so that
        * plaintext logs are readable. Also appends to sendable log.
        */
        public static void LogWarning(string text)
        {
            text = text.Replace("\n", "\n    ");

#if UNITY_EDITOR
            Debug.LogWarning(text);
#else
            Debug.LogFormat(LogType.Warning, LogOption.NoStacktrace, null, "{0}", text);
#endif
        }

        /**
        * Like Debug.LogError but only appends stacktrace if in editor so that
        * plaintext logs are readable. Also appends to sendable log.
        */
        public static void LogError(string text)
        {
            text = text.Replace("\n", "\n    ");

#if UNITY_EDITOR
            Debug.LogError(text);
#else
            Debug.LogFormat(LogType.Error, LogOption.NoStacktrace, null, "{0}", text);
#endif
        }
    }
}
