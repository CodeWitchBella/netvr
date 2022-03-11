using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json.Serialization;
using UnityEngine;

/**
 * This class contains data to be saved between restarts of the app
 *
 * Reading: IsblPersistentData.Instance.field
 *
 * Writing:
 * IsblPersistentData.Update((data) => {
 *   data.field = value;
 * });
 *
 * Adding new fields: just add a public member at the start.
 */
public sealed class IsblPersistentData : IIsblPersistentData
{
    public class Connection
    {
        [JsonPropertyName("peerId")]
        public UInt16 PeerId = 0;
        [JsonPropertyName("peerIdToken")]
        public string PeerIdToken = "";
        [JsonPropertyName("socketUrl")]
        public string SocketUrl = "";
    }

    [JsonPropertyName("connections")]
    public readonly List<Connection> Connections = new();

    public Connection GetConnection(string socketUrl)
        => Connections.Find(c => c.SocketUrl == socketUrl);

    public Connection GetLatestConnection() => Connections[^1];
    public int GetConnectionCount() => Connections.Count;
    public string GetConnectionSocketUrl(int id) => Connections[id].SocketUrl;

    public void AddConnection(string socketUrl, UInt16 peerId, string peerIdToken)
    {
        var existingIndex = Connections.FindIndex(conn => conn.SocketUrl == socketUrl);
        if (existingIndex == Connections.Count - 1 && existingIndex >= 0)
        {
            var existing = Connections[existingIndex];
            if (existing.PeerId == peerId && existing.PeerIdToken == peerIdToken) return; // No change needed
        }

        if (existingIndex >= 0) Connections.RemoveAt(existingIndex);

        Connections.Add(new Connection { SocketUrl = socketUrl, PeerId = peerId, PeerIdToken = peerIdToken });
    }

    [JsonPropertyName("logLocalData")]
    public bool LogLocalData;

    public void Init()
    {
        void InsertIfNotPresent(string url)
        {
            if (!Connections.Any(c => c.SocketUrl == url))
                Connections.Add(new Connection { SocketUrl = url });
        }

        InsertIfNotPresent("wss://netvr.isbl.workers.dev/");
        InsertIfNotPresent("ws://192.168.1.31:10000/");
    }

    static readonly IsblPersistentDataSaver<IsblPersistentData> _saver = new();
    public delegate void Updater(IsblPersistentData instance);
    public static void Update(Updater updater)
    {
        updater(_saver.Instance);
        _saver.Save();
    }

    public static IsblPersistentData Instance { get { return _saver.Instance; } }
}
