using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;
using UnityEngine;

namespace Isbl.NetVR
{

    internal static class Utils
    {

        private static string Truncate(string text)
        {
            if (text.Length < 16000)
            {
                return text;
            }
            text = text.Substring(0, 16000);
            var index = text.LastIndexOf('\n');
            if (index < 0) return text;
            return text.Substring(0, index);
        }

        /**
         * Like Debug.Log but only appends stacktrace if in editor so that plaintext
         * logs are readable. Also appends to senable log.
         */
        public static void Log(string text)
        {
            text = text.Replace("\n", "\n    ");

            while (text.Length > 0)
            {
                var to_print = Truncate(text);
                text = text.Substring(to_print.Length);
#if UNITY_EDITOR
                Debug.LogFormat(LogType.Log, LogOption.None, null, "{0}", to_print);
#else
                Debug.LogFormat(LogType.Log, LogOption.NoStacktrace, null, "{0}", to_print);
#endif
            }
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
