using System;
using UnityEngine;
using System.Linq;
using System.Collections.Generic;
using Newtonsoft.Json.Linq;
using System.Buffers.Binary;
using System.IO;
using System.IO.Compression;

/// <summary>
/// Main implementation of network interface
/// </summary>
/// Do not construct this directly, use .Instance accessor instead.
public sealed class IsblNet : IDisposable
{
    /// <summary>
    /// Makes sure that there is existing instance of IsblNet. This is usually
    /// not necessary to call directly because .Instance already does that.
    /// </summary>
    /// This method is used in Singleton instantiation scheme and is the only
    /// place where IsblNet constructor is called.
    public static void EnsureInstanceExists() { if (!IsblNetComponent.InstanceExists) IsblNetComponent.Instance = new(); }

    /// <summary>
    /// When you need to send or recieve message via network use this to access
    /// primary IsblNet instance
    /// </summary>
    public static IsblNet Instance { get { EnsureInstanceExists(); return IsblNetComponent.Instance; } }

    ReconnectingClientWebSocket _socket;
    public bool Connected => false;
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
                // cleanup
                _socket.Dispose();
            }
            // reinit
            var data = IsblPersistentData.Instance.GetConnection(value);
            Debug.Log($"Connecting to: {data.SocketUrl} with {data.PeerId}, {data.PeerIdToken}");
            LocalState.Id = data.PeerId;
            LocalState.IdToken = data.PeerIdToken;
            _socket = new(data.SocketUrl);
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
            if (LocalState.Id == 0) _socket.SendAsync(new { action = "gimme id" });
            else _socket.SendAsync(new { action = "i already has id", id = LocalState.Id, token = LocalState.IdToken });
        };
        _socket.OnDisconnect += () =>
        {
            LocalState.Initialized = false;
            OtherStates.Clear();
        };
        _socket.OnTextMessage += (text) =>
        {
            try
            {
                var obj = JObject.Parse(text);
                var action = obj.Value<string>("action");
                if (action == "id's here" || action == "id ack")
                {
                    if (action == "id's here")
                    {
                        LocalState.Id = obj.Value<int>("intValue");
                        LocalState.IdToken = obj.Value<string>("stringValue");
                    }
                    IsblPersistentData.Instance.SaveConnection(socketUrl: _socket.Uri.ToString(), peerId: LocalState.Id, peerIdToken: LocalState.IdToken);
                    LocalState.Initialized = true;
                    SendDeviceInfo();
                }
                else if (action == "device info")
                {
                    HandleDeviceInfo(obj);
                }
                else if (action == "set calibration")
                {
                    HandleSetCalibration(obj);
                }
                else if (action == "disconnect")
                {
                    foreach (var idToken in obj.GetValue("ids").Children<JToken>())
                    {
                        int id = (int)idToken;
                        if (OtherStates.ContainsKey(id)) OtherStates.Remove(id);
                    }
                }
                else if (!string.IsNullOrEmpty(action))
                {
                    Debug.Log($"Unknown action \"{action}\"");
                }
            }
            catch (Exception e)
            {
                Debug.LogException(e);
            }
        };
        _socket.OnBinaryMessage += (data) =>
            {
                int clientCount = BinaryPrimitives.ReadInt32LittleEndian(data);
                var offset = 4;

                for (int clientIndex = 0; clientIndex < clientCount; ++clientIndex)
                {
                    var clientId = BinaryPrimitives.ReadInt32LittleEndian(data[offset..]);
                    offset += 4;
                    offset += Isbl.NetData.Read7BitEncodedInt(data[offset..], out var numberOfDevices);

                    for (int deviceIndex = 0; deviceIndex < numberOfDevices; deviceIndex++)
                    {
                        HandleDeviceData(clientId, data, offset);
                        offset += Isbl.NetData.Read7BitEncodedInt(data[offset..], out var deviceBytes);
                        offset += deviceBytes;
                    }
                }
            };
    }

    void HandleDeviceData(int clientId, byte[] bytes, int offset)
    {
        var offsetCopy = offset + Isbl.NetData.Read7BitEncodedInt(bytes[offset..], out var deviceBytes);
        offsetCopy += Isbl.NetData.Read7BitEncodedInt(bytes[offsetCopy..], out var deviceId);

        var state = OtherStates.GetValueOrDefault(clientId, null);
        if (state != null)
        {
            var device = state.Devices.GetValueOrDefault(deviceId, null);
            device?.DeSerializeData(bytes, offsetCopy);
        }
    }

    void HandleDeviceInfo(JObject obj)
    {
        foreach (var peer in obj.GetValue("info").Children<JObject>())
        {
            var id = peer.Value<int>("id");
            Isbl.NetStateData localOtherState = OtherStates.GetValueOrDefault(id, null);
            if (localOtherState == null)
            {
                localOtherState = new()
                {
                    Id = id,
                    Initialized = true,
                };
                OtherStates.Add(id, localOtherState);
            }
            if (!peer.ContainsKey("info")) continue;

            var children = peer.GetValue("info").Children<JObject>();
            HashSet<int> unvisited = new(localOtherState.Devices.Keys);
            foreach (var device in children)
            {
                var localId = device.Value<int>("localId");
                unvisited.Remove(localId);
                var localDevice = localOtherState.Devices.GetValueOrDefault(localId, null);
                if (localDevice == null)
                {
                    localDevice = new();
                    localOtherState.Devices.Add(localId, localDevice);
                }
                localDevice.DeSerializeConfiguration(device);
            }
        }
    }

    /// <summary>
    /// Handles incoming messages with action set to "set calibration"
    /// </summary>
    void HandleSetCalibration(JObject obj)
    {
        foreach (var peer in obj.GetValue("calibrations").Children<JObject>())
        {
            var id = peer.Value<int>("id");
            Isbl.NetStateData localState = OtherStates.GetValueOrDefault(id, null);
            if (id == LocalState.Id)
            {
                LocalState.CalibrationPosition = Vector3Value(peer.Value<JObject>("translate"));
                LocalState.CalibrationRotation = Quaternion.Euler(Vector3Value(peer.Value<JObject>("rotate")) * 180f / (float)Math.PI);
                LocalState.CalibrationScale = Vector3Value(peer.Value<JObject>("scale"));
            }

            if (localState == null)
            {
                if (id != LocalState.Id)
                {
                    Debug.LogWarning($"Recieved set calibration message for unknown device id {id}. Did it arive before device info?");
                }
                continue;
            }

            localState.CalibrationPosition = Vector3Value(peer.Value<JObject>("translate"));
            localState.CalibrationRotation = Quaternion.Euler(Vector3Value(peer.Value<JObject>("rotate")) * 180f / (float)Math.PI);
            localState.CalibrationScale = Vector3Value(peer.Value<JObject>("scale"));
        }
    }

    /// <summary>
    /// Utility function used to deserialize Unity Vector3 from JSON in format
    /// {"x":1, "y":2, "z":3}
    /// </summary>
    Vector3 Vector3Value(JObject o)
    {
        return new Vector3(o.Value<float>("x"), o.Value<float>("y"), o.Value<float>("z"));
    }

    void SendDeviceInfo()
    {
        _ = _socket.SendAsync(new
        {
            action = "device info",
            deviceCount = LocalState.Devices.Count, // TODO remove this line
            info = (from d in LocalState.Devices where d.Value.HasData select d.Value.SerializeConfiguration()).ToArray(),
        });
        LocalState.DeviceInfoChanged = false;
    }

    public Isbl.NetStateData LocalState = new();
    public Dictionary<int, Isbl.NetStateData> OtherStates = new();

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
        if (!LocalState.Initialized || _socket == null) return;
        var bytes = new byte[LocalState.CalculateSerializationSize()];
        var span = bytes.AsSpan();
        BinaryPrimitives.WriteInt32LittleEndian(span[0..4], LocalState.Id);
        int offset = 4;
        offset += Isbl.NetData.Write7BitEncodedInt(span[offset..], LocalState.Devices.Count(d => d.Value.HasData));
        foreach (var devicePair in LocalState.Devices)
        {
            var device = devicePair.Value;
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
        if (LocalState.DeviceInfoChanged) SendDeviceInfo();
    }
}
