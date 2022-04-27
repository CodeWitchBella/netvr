using System;
using UnityEngine;
using System.Linq;
using System.Buffers.Binary;
using System.IO;
using System.IO.Compression;
using Json.Patch;
using System.Text.Json;
using System.Collections.Generic;

/// <summary>
/// Main implementation of network interface
/// </summary>
/// Do not construct this directly, use .Instance accessor instead.
public sealed class IsblNet : IDisposable
{
    public const int ProtocolVersion = 2;

    static IsblNet _instance;
    /// <summary>
    /// Utility to not require every component to have to search for IsblNet on Start()
    /// </summary>
    /// Also makes guards against accidental replacements.
    public static IsblNet Instance
    {
        get { return _instance; }
        set
        {
            if (value == null) _instance = value;
            else if (_instance != null) throw new Exception("Can't set multiple IsblNets");
            else _instance = value;
        }
    }

    public ReconnectingClientWebSocket Socket;

    JsonElement _serverStateJson;
    public Isbl.NetServerState ServerState = new();
    public Isbl.NetFastState FastState = new();
    public UInt16 SelfId = 0;
    public IsblLocalXRDeviceManager DeviceManager;

#if UNITY_EDITOR
    /// <summary>
    /// UNITY_EDITOR-only getter for _socket so that I don't have to implement
    /// readonly getter for everything I want to access from Editor GUI
    /// </summary>
    public ReconnectingClientWebSocket UnityEditorOnlyDebug { get { return Socket; } }
#endif

    public string SocketUrl
    {
        get { return Socket?.Uri.ToString() ?? ""; }
        set
        {
            if (SocketUrl == value) return;
            if (Socket != null)
            {
                // simulate disconnect
                Socket.OnDisconnect?.Invoke();
                Socket.OnDisconnect = null;
                // clean up
                Socket.Dispose();
            }
            // re-init
            Debug.Log($"Connecting to: {value}");
            Socket = new(value);
            InitializeSocket();
        }
    }

    public IsblNet()
    {
        var data = IsblPersistentData.Instance.GetLatestConnection();
        SocketUrl = data.SocketUrl;
    }

    readonly Dictionary<string, IIsblNetFeature> _features = new() {
        { CalibrationFeature.FeatureId, new CalibrationFeature() }
    };

    void InitializeSocket()
    {
        Socket.OnConnect += () =>
        {

            var conn = IsblPersistentData.Instance.GetConnection(SocketUrl);
            if (conn?.PeerId > 0)
                Socket.SendAsync(new { action = "i already has id", id = conn.PeerId, token = conn.PeerIdToken, protocolVersion = ProtocolVersion });
            else
                Socket.SendAsync(new { action = "gimme id", protocolVersion = ProtocolVersion });

            foreach (var feature in _features.Values) feature.Reset();
        };

        Socket.OnDisconnect += () =>
        {
            SelfId = 0;

            ServerState = new();
            FastState = new();
        };

        Socket.OnTextMessage += (text) =>
        {
            Debug.Log($"message: {text}");
            try
            {
                var obj = Newtonsoft.Json.Linq.JObject.Parse(text);
                var node = JsonDocument.Parse(text).RootElement;
                if (node.TryGetProperty("feature", out var featureElement))
                {
                    var featureId = featureElement.GetString();
                    if (String.IsNullOrEmpty(featureId)) throw new Exception("Expected feature to be string");
                    if (_features.TryGetValue(featureId, out var featureObject))
                    {
                        featureObject.OnMessage(this, node);
                    }
                    else
                    {
                        Debug.LogWarning($"Received message with unknown feature: \"{featureId}\"");
                    }
                }
                else if (node.TryGetProperty("action", out var actionElement))
                {
                    var action = actionElement.GetString();
                    if (action == "id's here" || action == "id ack")
                    {
                        var serverVersion = obj.Value<int>("protocolVersion");
                        if (serverVersion != ProtocolVersion)
                        {
                            throw new Exception($"Protocol version mismatch. Client: {ProtocolVersion}, Server: {serverVersion}.");
                        }

                        if (action == "id's here")
                        {
                            SelfId = obj.Value<UInt16>("intValue");
                            var idToken = obj.Value<string>("stringValue");
                            IsblPersistentData.Update((data) => data.AddConnection(
                                socketUrl: SocketUrl,
                                peerId: SelfId,
                                peerIdToken: idToken
                            ));
                        }
                        else
                        {
                            var conn = IsblPersistentData.Instance.GetConnection(SocketUrl);
                            SelfId = conn.PeerId;
                        }

                        SendDeviceInfo();
                    }
                    else if (action == "patch")
                    {
                        var patchString = Newtonsoft.Json.JsonConvert.SerializeObject(obj.Value<Newtonsoft.Json.Linq.JArray>("patches"));
                        var patch = JsonSerializer.Deserialize<JsonPatch>(patchString);
                        if (_serverStateJson.ValueKind != JsonValueKind.Object)
                            _serverStateJson = Isbl.NetUtils.JsonFromObject(ServerState); ;

                        var result = patch.Apply(_serverStateJson);
                        _serverStateJson = result.Result;
                        ServerState = JsonSerializer.Deserialize<Isbl.NetServerState>(JsonSerializer.Serialize(_serverStateJson));
                        UpdateFastStateStructure();
                    }
                    else if (action == "full state reset")
                    {
                        _serverStateJson = node.GetProperty("state");
                        ServerState = JsonSerializer.Deserialize<Isbl.NetServerState>(JsonSerializer.Serialize(_serverStateJson));
                        Debug.Log($"ServerState: {JsonSerializer.Serialize(ServerState, new JsonSerializerOptions { WriteIndented = true })}\njson: {_serverStateJson}");
                        UpdateFastStateStructure();
                    }
                    else
                    {
                        Debug.Log($"Unknown action \"{actionElement}\"");
                    }
                }
                else
                {
                    if (node.TryGetProperty("error", out var error))
                    {
                        Debug.LogError($"Server reported error: {error.GetString()}");
                    }
                    else
                    {
                        var json = JsonSerializer.Serialize(node, new JsonSerializerOptions { WriteIndented = true });
                        Debug.LogWarning($"Unknown message: {json}");
                    }
                }
            }
            catch (Exception e)
            {
                Debug.Log($"Error while processing text message:\n{text}");
                Debug.LogException(e);
            }
        };
        Socket.OnBinaryMessage += (data) =>
            {
                if (data.Length < 1) return;
                byte messageType = data[0];
                if (messageType == 1)
                {
                    int clientCount = BinaryPrimitives.ReadInt32LittleEndian(data[1..]);
                    var offset = 5;

                    for (int clientIndex = 0; clientIndex < clientCount; ++clientIndex)
                    {
                        var clientId = Isbl.NetUtils.CastUInt16(BinaryPrimitives.ReadInt32LittleEndian(data[offset..]));
                        offset += 4;
                        offset += Isbl.NetUtils.Read7BitEncodedInt(data[offset..], out var numberOfDevices);

                        for (int deviceIndex = 0; deviceIndex < numberOfDevices; deviceIndex++)
                        {
                            HandleDeviceData(clientId, data, offset);
                            offset += Isbl.NetUtils.Read7BitEncodedInt(data[offset..], out var deviceBytes);
                            offset += deviceBytes;
                        }
                    }
                }
                else if (messageType == 2)
                {
                    HandleHapticRequest(data, 1);
                }
                else
                {
                    Debug.LogWarning($"Unknown binary message type {messageType}");
                }
            };
    }

    /**
     * Takes data from server state and updates fast state's structure accordingly
     */
    void UpdateFastStateStructure()
    {
        var outdated = FastState.Clients.Keys.Where(fastStateClientId => !ServerState.Clients.ContainsKey(fastStateClientId)).ToArray();
        foreach (var key in outdated) FastState.Clients.Remove(key);

        foreach (var serverStateClientEntry in ServerState.Clients)
        {
            if (!FastState.Clients.TryGetValue(serverStateClientEntry.Key, out var fastClient))
            {
                fastClient = new();
                FastState.Clients.Add(serverStateClientEntry.Key, fastClient);
            }

            if (serverStateClientEntry.Value.Devices != null)
            {
                foreach (var deviceJson in serverStateClientEntry.Value.Devices)
                {
                    var id = deviceJson.GetProperty("localId").GetUInt16();
                    if (!fastClient.TryGetValue(id, out var fastDevice))
                    {
                        fastDevice = new() { DeviceData = new() };
                        fastClient.Add(id, fastDevice);
                    }

                    // TODO: remove Newtonsoft.Json here
                    var newtonsoftJson = Newtonsoft.Json.Linq.JObject.Parse(JsonSerializer.Serialize(deviceJson));
                    fastDevice.DeviceData.DeSerializeConfiguration(newtonsoftJson);
                }
            }
        }
    }

    void HandleHapticRequest(byte[] data, int offset)
    {
        Debug.Log($"HandleHapticRequest {string.Join(" ", data)}");
        while (offset < data.Length)
        {
            var deviceId = BinaryPrimitives.ReadUInt16LittleEndian(data[offset..]);
            // TODO: update offset+= and remove this this once message format uses UInt16
            if (BinaryPrimitives.ReadUInt16LittleEndian(data[(offset + 2)..]) != 0)
                throw new Exception("Id is > 65535");

            offset += 4;
            uint channel = BinaryPrimitives.ReadUInt32LittleEndian(data[offset..]);
            offset += 4;
            BinaryReader reader = new BinaryReader(new MemoryStream(data[offset..]));
            float duration = reader.ReadSingle();
            float amplitude = reader.ReadSingle();
            offset += 8;
            //Debug.Log($"Net Haptic. deviceId: {deviceId}, channel: {channel}, amplitude: {amplitude}, duration: {duration}.");
            if (DeviceManager.TryFindDevice(deviceId, out var device))
            {
                device.LocalDevice.Device.SendHapticImpulse(channel, amplitude, duration);
            }
        }
    }

    void HandleDeviceData(UInt16 clientId, byte[] bytes, int offset)
    {
        var offsetCopy = offset + Isbl.NetUtils.Read7BitEncodedInt(bytes[offset..], out var deviceBytes);
        offsetCopy += Isbl.NetUtils.Read7BitEncodedInt(bytes[offsetCopy..], out var deviceId);

        if (FastState.TryGetRemoteDevice(clientId, Isbl.NetUtils.CastUInt16(deviceId), out var device))
        {
            device.DeviceData.DeSerializeData(bytes, offsetCopy);
        }
    }

    void SendDeviceInfo()
    {
        Debug.Log("Sending device info");
        DeviceManager.DeviceInfoChanged = false;
        _ = Socket.SendAsync(new
        {
            action = "set",
            client = SelfId,
            field = "devices",
            value = (from d in DeviceManager.Devices where d.NetDevice.HasData select d.NetDevice.SerializeConfiguration()).ToArray(),
        });
    }

    public void Dispose()
    {
        Socket?.Dispose();
        if (Instance == this) Instance = null;
    }

#if UNITY_EDITOR
    public StatsData Stats;
    public struct StatsData
    {
        public int MessageSize;
        public int MessageSizeGzip;
        public int MessageSizeBrotli;
        public int MessageSizeBrotliMax;
    };

    static byte[] CompressGzip(byte[] data)
    {
        using var compressedStream = new MemoryStream();
        using (var zipStream = new GZipStream(compressedStream, System.IO.Compression.CompressionLevel.Optimal))
        { zipStream.Write(data, 0, data.Length); }
        return compressedStream.ToArray();
    }

    static byte[] CompressBrotli(byte[] data)
    {
        using var compressedStream = new MemoryStream();
        using (var zipStream = new BrotliStream(compressedStream, System.IO.Compression.CompressionLevel.Optimal))
        { zipStream.Write(data, 0, data.Length); }
        return compressedStream.ToArray();
    }
#endif

    /// <summary>
    /// Periodically called from IsblNetComponent to upload current LocalState
    /// to server
    /// </summary>
    public void Tick()
    {
        if (SelfId < 1 || Socket == null) return;
        var bytes = new byte[DeviceManager.CalculateSerializationSize() + 1];
        var span = bytes.AsSpan();
        span[0] = 1;
        //BinaryPrimitives.WriteInt32LittleEndian(span[1..5], LocalState.Id);
        int offset = 1;
        offset += Isbl.NetUtils.Write7BitEncodedInt(span[offset..], DeviceManager.Devices.Count(d => d.NetDevice.HasData));
        foreach (var driver in DeviceManager.Devices)
        {
            var device = driver.NetDevice;
            if (!device.HasData) continue;
            var len = device.CalculateSerializationSize();
            var realLen = device.SerializeData(bytes, offset);
            if (len != realLen) Debug.LogWarning($"Length mismatch: theoretical {len} vs real {realLen}");
            offset += len;
        }
        if (offset != bytes.Length) Debug.LogWarning($"Wrong final offset: theoretical {bytes.Length} vs real {offset}");
#if UNITY_EDITOR
        Stats.MessageSize = bytes.Length;
        Stats.MessageSizeGzip = CompressGzip(bytes).Length;
        Stats.MessageSizeBrotli = CompressBrotli(bytes).Length;
        Stats.MessageSizeBrotliMax = Math.Max(Stats.MessageSizeBrotliMax, Stats.MessageSizeBrotli);
#endif

        _ = Socket.SendAsync(bytes, System.Net.WebSockets.WebSocketMessageType.Binary);
        if (DeviceManager.DeviceInfoChanged) SendDeviceInfo();

        foreach (var feature in _features.Values) feature.Tick(this);
    }
}
