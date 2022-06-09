using System;
using System.IO;
using System.Runtime.InteropServices;
using UnityEngine;

/// <summary>
/// Wrapper around my native dynamic library managing its loading and unloading
/// (when supported)
/// </summary>
class IsblDynamicLibrary : IDisposable
{
#if UNITY_EDITOR_WIN
    /// <summary>
    /// Windows-specific functions for dll loading and unloading so that I dont
    /// have to restart Unity Editor every time I change the dll.
    /// </summary>
    static class SystemLibrary
    {
        [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Ansi)]
        static public extern IntPtr LoadLibrary(string lpFileName);

        [DllImport("kernel32", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        static public extern bool FreeLibrary(IntPtr hModule);

        [DllImport("kernel32", CharSet = CharSet.Ansi, ExactSpelling = true, SetLastError = true)]
        static public extern IntPtr GetProcAddress(IntPtr hModule, string procedureName);

        static public void GetDelegate<TDelegate>(IntPtr lib, string name, out TDelegate val)
        {
            var addr = GetProcAddress(lib, name);
            if (addr == default)
            {
                Utils.LogWarning($"Can't retrieve address of {name}");
                val = default;
            }
            else
            {
                val = Marshal.GetDelegateForFunctionPointer<TDelegate>(addr);
            }
        }
    }
    IntPtr _library;
    readonly string _fullPath;

    public void GetDelegate<TDelegate>(string name, out TDelegate val)
    {
        var addr = SystemLibrary.GetProcAddress(_library, name);
        if (addr == default)
        {
            Utils.LogWarning($"Can't retrieve address of {name}");
            val = default;
        }
        else
        {
            val = Marshal.GetDelegateForFunctionPointer<TDelegate>(addr);
        }
    }
#else
    public void GetDelegate<TDelegate>(string name, out TDelegate val)
    {
        throw new Exception("This function is only available on UNITY_EDITOR_WIN");
    }
#endif // UNITY_EDITOR_WIN

    /**
     * Same as File.OpenRead but returns null if file was not found instead of
     * throwing exception.
     */
    static FileStream SafeOpenRead(string path)
    {
        try { return File.OpenRead(path); }
        catch (FileNotFoundException) { return null; }
    }

    /**
     * Compares contents of given files.
     */
    static bool AreFilesIdentical(string path1, string path2)
    {
        using var fs1 = SafeOpenRead(path1);
        using var fs2 = SafeOpenRead(path2);
        if (fs1 == null || fs2 == null) return fs1 == fs2;

        if (fs1.Length != fs2.Length) return false;

        var fs1buf = new BufferedStream(fs1);
        var fs2buf = new BufferedStream(fs2);
        while (true)
        {
            // Read one byte from each file.
            var file1byte = fs1buf.ReadByte();
            var file2byte = fs2buf.ReadByte();
            if (file1byte == -1) return true;
            if (file1byte != file2byte) return false;
        }
    }

    public IsblDynamicLibrary(string name, string prefix)
    {
#if UNITY_EDITOR_WIN
        for (int i = 0; ; ++i)
        {
            _fullPath = prefix + name + i;
            try
            {
                using (var watch = new IsblStopwatch("AreFilesIdentical:.dll"))
                    if (AreFilesIdentical(prefix + name + ".dll", _fullPath + ".dll") && AreFilesIdentical(prefix + name + ".pdb", _fullPath + ".pdb")) break;
                // copy to new file so that original is still writeable
                File.Copy(prefix + name + ".dll", _fullPath + ".dll", true);
                try
                {
                    File.Copy(prefix + name + ".pdb", _fullPath + ".pdb", true);
                }
                catch (IOException) { }
                break;
            }
            catch (IOException) when (i < 9)
            {
                // retry
            }
            catch (IOException e)
            {
                Utils.LogException(e);
                break;
            }
        }

        // load
        Utils.Log($"Loading library from {_fullPath}.dll");
        _library = SystemLibrary.LoadLibrary(_fullPath + ".dll");
        if (_library == default) Utils.LogWarning($"Failed to load {_fullPath}.dll");
#endif 
    }

    public void Dispose()
    {
#if UNITY_EDITOR_WIN
        if (_library != default)
        {
            Utils.Log("IsblDynamicLibrary.Dispose()");
            SystemLibrary.FreeLibrary(_library);
            _library = default;
            try
            {
                File.Delete(_fullPath + ".dll");
                File.Delete(_fullPath + ".dll.meta");
            }
            catch (IOException e) { Utils.LogException(e); }
            try
            {
                File.Delete(_fullPath + ".pdb");
                File.Delete(_fullPath + ".pdb.meta");
            }
            catch (IOException) { }
        }
#endif
    }

    public const bool DoesUnload =
#if UNITY_EDITOR_WIN
        true
#else
        false
#endif
;
}
