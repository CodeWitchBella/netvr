using System;
using UnityEngine;
using System.Linq;
using System.Collections.Generic;
using System.Buffers.Binary;
using System.IO;
using System.IO.Compression;
using System.Text.Json.Nodes;
using Json.Patch;
using System.Text.Json;

/// <summary>
/// Main implementation of network interface
/// </summary>
/// Do not construct this directly, use .Instance accessor instead.
public sealed class IsblNet : IDisposable
{
    public const int ProtocolVersion = 2;

    /// <summary>
    /// Makes sure that there is existing instance of IsblNet. This is usually
    /// not necessary to call directly because .Instance already does that.
    /// </summary>
    /// This method is used in Singleton instantiation scheme and is the only
    /// place where IsblNet constructor is called.
    public static void EnsureInstanceExists() { if (!IsblNetComponent.InstanceExists) IsblNetComponent.Instance = new(); }

    /// <summary>
    /// When you need to send or receive message via network use this to access
    /// primary IsblNet instance
    /// </summary>
    public static IsblNet Instance { get { EnsureInstanceExists(); return IsblNetComponent.Instance; } }

    ReconnectingClientWebSocket _socket;

    JsonNode _serverStateJson;
    public Isbl.NetServerState ServerState = new();
    public Isbl.NetFastState FastState = new();
    public UInt16 SelfId = 0;
    public IsblLocalXRDeviceManager DeviceManager;

#if UNITY_EDITOR
    /// <summary>
    /// UNITY_EDITOR-only getter for _socket so that I don't have to implement
    /// readonly getter for everything I want to access from Editor GUI
    /// </summary>
    public ReconnectingClientWebSocket UnityEditorOnlyDebug { get { return _socket; } }
#endif

    public string SocketUrl
    {
        get { return _socket?.Uri.ToString() ?? ""; }
        set
        {
            if (SocketUrl == value) return;
            if (_socket != null)
            {
                // simulate disconnect
                _socket.OnDisconnect?.Invoke();
                _socket.OnDisconnect = null;
                // clean up
                _socket.Dispose();
            }
            // re-init
            Debug.Log($"Connecting to: {value}");
            _socket = new(value);
            InitializeSocket();
        }
    }

    /// <summary>Private constructor. Only place where this is called is from
    /// EnsureInstanceExists and that only happens if other instance does not
    /// exist already.</summary>
    IsblNet()
    {
        var data = IsblPersistentData.Instance.GetLatestConnection();
        SocketUrl = data.SocketUrl;
    }

    void InitializeSocket()
    {
        _socket.OnConnect += () =>
        {
            var conn = IsblPersistentData.Instance.GetConnection(SocketUrl);
            if (conn != null)
                _socket.SendAsync(new { action = "i already has id", id = conn.PeerId, token = conn.PeerIdToken, protocolVersion = ProtocolVersion });
            else
                _socket.SendAsync(new { action = "gimme id", protocolVersion = ProtocolVersion });
        };

        _socket.OnDisconnect += () =>
        {
            SelfId = 0;

            ServerState = new();
            FastState = new();
        };

        _socket.OnTextMessage += (text) =>
        {
            Debug.Log($"message: {text}");
            try
            {
                var obj = Newtonsoft.Json.Linq.JObject.Parse(text);
                var action = obj.Value<string>("action");
                if (action == null)
                {
                    JsonNode node = JsonNode.Parse(text);
                    if (node["error"] != null)
                    {
                        Debug.LogError($"Server reported error: {node["error"].GetValue<string>()}");
                    }
                    else
                    {
                        var json = JsonSerializer.Serialize(node, new JsonSerializerOptions { WriteIndented = true });
                        Debug.LogWarning($"Unknown message: {json}");
                    }
                }
                else if (action == "id's here" || action == "id ack")
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
                    if (_serverStateJson == null)
                        _serverStateJson = JsonNode.Parse(JsonSerializer.Serialize(ServerState));

                    _serverStateJson = patch.Apply(_serverStateJson);
                    ServerState = _serverStateJson.Deserialize<Isbl.NetServerState>();
                }
                else if (action == "full state reset")
                {
                    JsonNode node = JsonNode.Parse(text);
                    _serverStateJson = node["state"];
                    ServerState = _serverStateJson.Deserialize<Isbl.NetServerState>();
                    Debug.Log($"ServerState: {JsonSerializer.Serialize(ServerState, new JsonSerializerOptions { WriteIndented = true })}\njson: {node["state"]}");
                }
                else if (!string.IsNullOrEmpty(action))
                {
                    Debug.Log($"Unknown action \"{action}\"");
                }
            }
            catch (Exception e)
            {
                Debug.Log($"Error while processing text message:\n{text}");
                Debug.LogException(e);
            }
        };
        _socket.OnBinaryMessage += (data) =>
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
        _ = _socket.SendAsync(new
        {
            action = "set",
            client = SelfId,
            field = "devices",
            value = (from d in DeviceManager.Devices where d.NetDevice.HasData select d.NetDevice.SerializeConfiguration()).ToArray(),
        });
        DeviceManager.DeviceInfoChanged = false;
    }

    public void Dispose()
    {
        _socket?.Dispose();
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
        if (SelfId < 1 || _socket == null) return;
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

        _ = _socket.SendAsync(bytes, System.Net.WebSockets.WebSocketMessageType.Binary);
        if (DeviceManager.DeviceInfoChanged) SendDeviceInfo();
    }
}
