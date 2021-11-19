using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using Newtonsoft.Json;
using UnityEngine;

public sealed class IsblPersistentData
{
    static IsblPersistentData _instance;
    public static string DataPath
    {
        get
        {
            if (Application.platform == RuntimePlatform.Android) Path.Combine(Application.persistentDataPath, "isbl-persistent-data.json");
            return Path.Combine(Directory.GetCurrentDirectory(), "isbl-persistent-data.json");
        }
    }

    public static IsblPersistentData Instance
    {
        get
        {
            if (_instance == null)
            {
                try
                {
                    using var reader = new StreamReader(DataPath);
                    _instance = JsonConvert.DeserializeObject<IsblPersistentData>(reader.ReadToEnd());
                }
                catch
                {
                    _instance = new();
                }
            }
            return _instance;
        }
    }

    public class Connection
    {
        [JsonProperty(propertyName: "peerId")]
        public int PeerId;
        [JsonProperty(propertyName: "peerIdToken")]
        public string PeerIdToken;
        [JsonProperty(propertyName: "socketUrl")]
        public string SocketUrl;
    }

    [JsonProperty(propertyName: "connections")]
    readonly List<Connection> _connections = new();

    public Connection GetLatestConnection()
    {
        if (_connections.Count == 0)
        {
            return new Connection
            {
                PeerId = 0,
                PeerIdToken = "",
                SocketUrl = "wss://netvr.isbl.workers.dev",
            };
        }
        return _connections[^1];
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

        using var writer = new StreamWriter(DataPath);
        await writer.WriteAsync(JsonConvert.SerializeObject(this));
        _saving = false;

        if (_shouldResave) Save();
    }

    IsblPersistentData() { }
}
