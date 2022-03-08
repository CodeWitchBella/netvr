using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using UnityEngine;

public sealed class IsblPersistentData
{
    static IsblPersistentData _instance;
    public static string DataDirectory
    {
        get
        {
            if (Application.platform == RuntimePlatform.Android) return Path.Combine(Application.persistentDataPath, "user-data");
            return Path.Combine(Directory.GetCurrentDirectory(), "user-data");
        }
    }
    static string DataPath => Path.Combine(DataDirectory, "config.json");

    public static IsblPersistentData Instance
    {
        get
        {
            if (_instance == null)
            {
                var fileData = "";
                try
                {
                    Debug.Log($"Application.persistentDataPath: {Application.persistentDataPath}, Current Directory: {Directory.GetCurrentDirectory()}");
                    using var reader = new StreamReader(DataPath);
                    fileData = reader.ReadToEnd();
                    _instance = Newtonsoft.Json.JsonConvert.DeserializeObject<IsblPersistentData>(fileData);
                }
                catch
                {
                    _instance = new();
                }
                if (_instance._connections.Count < 1)
                {
                    _instance._connections.Add(new Connection { SocketUrl = "wss://netvr.isbl.workers.dev/" });
                    _instance._connections.Add(new Connection { SocketUrl = "ws://192.168.1.31:10000/" });
                }

                if (Newtonsoft.Json.JsonConvert.SerializeObject(_instance) != fileData) _instance.Save();
            }
            return _instance;
        }
    }

    public class Connection
    {
        [Newtonsoft.Json.JsonProperty(propertyName: "peerId")]
        public int PeerId = 0;
        [Newtonsoft.Json.JsonProperty(propertyName: "peerIdToken")]
        public string PeerIdToken = "";
        [Newtonsoft.Json.JsonProperty(propertyName: "socketUrl")]
        public string SocketUrl;
    }

    [Newtonsoft.Json.JsonProperty(propertyName: "connections")]
    readonly List<Connection> _connections = new();

    bool _logLocalData;

    [Newtonsoft.Json.JsonProperty(propertyName: "logLocalData")]
    public bool LogLocalData
    {
        get => _logLocalData;
        set
        {
            _logLocalData = value;
            Save();
        }
    }

    public Connection GetLatestConnection()
    {
        return _connections[^1];
    }

    public string GetConnectionSocketUrl(int id)
    {
        return _connections[id]?.SocketUrl ?? "<none>";
    }

    public int GetConnectionCount()
    {
        return _connections.Count;
    }

    public Connection GetConnection(string socketUrl)
    {
        socketUrl = new System.Uri(socketUrl).ToString(); // normalize
        var con = _connections.Find(c => c.SocketUrl == socketUrl);
        return con ?? new Connection
        {
            PeerId = 0,
            PeerIdToken = "",
            SocketUrl = socketUrl,
        };
    }

    public void SaveConnection(string socketUrl, int peerId, string peerIdToken)
    {
        var existingIndex = _connections.FindIndex(conn => conn.SocketUrl == socketUrl);
        if (existingIndex == _connections.Count - 1 && existingIndex >= 0)
        {
            var existing = _connections[existingIndex];
            if (existing.PeerId == peerId && existing.PeerIdToken == peerIdToken) return; // No change needed
        }

        if (existingIndex >= 0) _connections.RemoveAt(existingIndex);

        _connections.Add(new Connection { SocketUrl = socketUrl, PeerId = peerId, PeerIdToken = peerIdToken });
        Save();
    }

    bool _saving;
    bool _shouldResave;
    void Save()
    {
        if (_saving) _shouldResave = true;
        else _ = SaveInternal();
    }

    async Task SaveInternal()
    {
        _saving = true;
        await Task.Delay(10);
        _shouldResave = false;

        Debug.Log($"Writing PersistentData to {DataPath}");
        Directory.CreateDirectory(DataDirectory);
        {
            using var writer = new StreamWriter(DataPath);
            await writer.WriteAsync(Newtonsoft.Json.JsonConvert.SerializeObject(this));
        }
        _saving = false;

        if (_shouldResave) Save();
    }

    IsblPersistentData() { }
}
