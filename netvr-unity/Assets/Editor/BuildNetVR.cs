using System.IO;
using UnityEditor;
using UnityEditor.Build.Reporting;
using System.Linq;
using UnityEngine;
using System.Collections.Generic;
using System;

public static class BuildNetVR
{
    [MenuItem("Build/Android Build")]
    public static void BuildAndroid()
    {
        ParseCommandLineArguments(out var options);
        if (options.ContainsKey("buildVersion")) PlayerSettings.bundleVersion = options["buildVersion"];
        if (options.ContainsKey("androidVersionCode")) PlayerSettings.Android.bundleVersionCode = int.Parse(options["androidVersionCode"]);

        Build(BuildTarget.Android, Path.Combine("android", "netvr.apk"));
    }

    [MenuItem("Build/Windows Build")]
    public static void BuildWindows64()
    {
        Build(BuildTarget.StandaloneWindows64, Path.Combine("windows", "netvr.exe"));
    }

    static void Build(BuildTarget buildTarget, string filePath)
    {
        string[] scenes = EditorBuildSettings.scenes.Where(scene => scene.enabled).Select(s => s.path).ToArray();
        var buildPlayerOptions = new BuildPlayerOptions
        {
            scenes = scenes,
            locationPathName = Path.Combine("..", "netvr-unity-builds", filePath),
            target = buildTarget,
        };
        BuildReport buildReport = BuildPipeline.BuildPlayer(buildPlayerOptions);
        var summary = buildReport.summary;
    }

    // copied from https://github.com/game-ci/documentation/blob/main/example/BuildScript.cs
    static void ParseCommandLineArguments(out Dictionary<string, string> providedArguments)
    {
        providedArguments = new Dictionary<string, string>();
        string[] args = Environment.GetCommandLineArgs();

        // Extract flags with optional values
        for (int current = 0, next = 1; current < args.Length; current++, next++)
        {
            // Parse flag
            bool isFlag = args[current].StartsWith("-");
            if (!isFlag) continue;
            string flag = args[current].TrimStart('-');

            // Parse optional value
            bool flagHasValue = next < args.Length && !args[next].StartsWith("-");
            string value = flagHasValue ? args[next].TrimStart('-') : "";
            string displayValue = flag.Contains("key", StringComparison.OrdinalIgnoreCase) ? "*HIDDEN*" : "\"" + value + "\"";

            // Assign
            Console.WriteLine($"Found flag \"{flag}\" with value {displayValue}.");
            providedArguments.Add(flag, value);
        }
    }
}
