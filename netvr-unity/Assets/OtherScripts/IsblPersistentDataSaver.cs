using System;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using UnityEngine;

public interface IIsblPersistentData
{
    void Init();
}

/**
 * This class contains implementation of IsblPersistentData singleton mechanism.
 * It's separate so that it's easier to see what is going on without looking at
 * the particular data being saved.
 */
public sealed class IsblPersistentDataSaver<TInstance> where TInstance : IIsblPersistentData, new()
{
    TInstance _instance;
    static string _fileData;

    public static string DataDirectory
    {
        get
        {
            if (Application.platform == RuntimePlatform.Android) return Path.Combine(Application.persistentDataPath, "user-data");
            return Path.Combine(Directory.GetCurrentDirectory(), "user-data");
        }
    }
    static string DataPath => Path.Combine(DataDirectory, "config.json");

    string Serialize()
    {
        return JsonSerializer.Serialize(_instance, new JsonSerializerOptions
        {
            IncludeFields = true,
            IgnoreReadOnlyFields = false,
            WriteIndented = true,
        });
    }

    public TInstance Instance
    {
        get
        {
            if (_instance == null)
            {
                try
                {
                    Debug.Log($"DataPath: {DataPath}\nDataDirectory: {DataDirectory}\nApplication.persistentDataPath: {Application.persistentDataPath}\nCurrent Directory: {Directory.GetCurrentDirectory()}");
                    using var reader = new StreamReader(DataPath);
                    _fileData = reader.ReadToEnd();
                    _instance = JsonSerializer.Deserialize<TInstance>(_fileData) ?? new TInstance();
                }
                catch
                {
                    _instance = new TInstance();
                }
                _instance.Init();

                if (Serialize() != _fileData) Save();
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
            var value = Serialize();
            if (value != _fileData)
            {
                await Task.Run(() => Directory.CreateDirectory(DataDirectory));
                using var writer = new StreamWriter(DataPath);
                await writer.WriteAsync(value);
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

    public IsblPersistentDataSaver() { }
}
