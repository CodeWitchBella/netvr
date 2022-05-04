using System;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using UnityEngine;

namespace Isbl.Persistent
{
    public interface IData
    {
        void Init();
        string Serialize();
    }

    public sealed class DataDirectory
    {
        public static string Name
        {
            get
            {
                if (Application.platform == RuntimePlatform.Android) return Path.Combine(Application.persistentDataPath, "user-data");
                return Path.Combine(System.IO.Directory.GetCurrentDirectory(), "user-data");
            }
        }

        public static Task WriteFile(string fileName, object value)
        {
            return WriteFile(fileName, JsonSerializer.Serialize(value, new JsonSerializerOptions
            {
                IncludeFields = true,
                IgnoreReadOnlyFields = false,
                WriteIndented = true,
            }));
        }

        public async static Task WriteFile(string fileName, string value)
        {
            string dataDir = Name;
            string dataPath = Path.Combine(dataDir, fileName);
            await Task.Run(() => System.IO.Directory.CreateDirectory(dataDir));
            using var writer = new StreamWriter(dataPath);
            await writer.WriteAsync(value);
        }

        public static string ReadFileSync(string fileName)
        {
            string dataDir = Name;
            string dataPath = Path.Combine(dataDir, fileName);
            using var reader = new StreamReader(dataPath);
            return reader.ReadToEnd();
        }
    }

    /**
     * This class contains implementation of IsblPersistentData singleton mechanism.
     * It's separate so that it's easier to see what is going on without looking at
     * the particular data being saved.
     */
    public sealed class Saver<TInstance> where TInstance : IData, new()
    {
        TInstance _instance;
        static string _fileData;

        readonly string _fileName;

        public Saver(string fileName)
        {
            _fileName = fileName;
        }

        public TInstance Instance
        {
            get
            {
                if (_instance == null)
                {
                    try
                    {
                        Debug.Log($"fileName: {_fileName}\nDataDirectory: {DataDirectory.Name}\nApplication.persistentDataPath: {Application.persistentDataPath}\nCurrent Directory: {Directory.GetCurrentDirectory()}");
                        _fileData = DataDirectory.ReadFileSync(_fileName);
                        _instance = JsonSerializer.Deserialize<TInstance>(_fileData) ?? new TInstance();
                    }
                    catch
                    {
                        _instance = new TInstance();
                    }
                    _instance.Init();

                    if (_instance.Serialize() != _fileData) Save();
                }
                return _instance;
            }
        }

        bool _saving;
        bool _shouldResave;
        public void Save()
        {
            if (_saving) _shouldResave = true;
            else _ = SaveInternal();
        }

        async Task SaveInternal()
        {
            _saving = true;
            _shouldResave = false;
            try
            {
                var value = _instance.Serialize();
                if (value != _fileData)
                {
                    await DataDirectory.WriteFile(_fileName, value);
                    _fileData = value;
                }
                await Task.Delay(10);
            }
            catch (Exception e)
            {
                Debug.LogError(e);
            }
            _saving = false;

            if (_shouldResave) Save();
        }

        public Saver() { }
    }
}
