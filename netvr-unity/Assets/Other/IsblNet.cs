using System;
using UnityEngine;
using System.Linq;
using System.Collections.Generic;
using Newtonsoft.Json.Linq;
using System.Buffers.Binary;

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
            NetState.Id = data.PeerId;
            NetState.IdToken = data.PeerIdToken;
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
            if (NetState.Id == 0) _socket.SendAsync(new { action = "gimme id" });
            else _socket.SendAsync(new { action = "i already has id", id = NetState.Id, token = NetState.IdToken });
        };
        _socket.OnDisconnect += () =>
        {
            NetState.Initialized = false;
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
                        NetState.Id = obj.Value<int>("intValue");
                        NetState.IdToken = obj.Value<string>("stringValue");
                    }
                    IsblPersistentData.Instance.SaveConnection(socketUrl: _socket.Uri.ToString(), peerId: NetState.Id, peerIdToken: NetState.IdToken);
                    NetState.Initialized = true;
                    SendDeviceInfo();
                }
                else if (action == "device info")
                {
                    HandleDeviceInfo(obj);
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
            var device = state.Devices.Find(dev => dev.LocallyUniqueId == deviceId);
            device?.DeSerializeData(bytes, offsetCopy);
        }
    }

    void HandleDeviceInfo(JObject obj)
    {
        foreach (var peer in obj.GetValue("info").Children<JObject>())
        {
            var id = peer.Value<int>("id");
            Isbl.NetStateData localState = OtherStates.GetValueOrDefault(id, null);
            if (localState == null)
            {
                localState = new()
                {
                    Id = id,
                    Initialized = true,
                };
                OtherStates.Add(id, localState);
            }
            if (!peer.ContainsKey("info")) continue;

            var children = peer.GetValue("info").Children<JObject>();
            localState.ResizeDevices(children.Count());
            var i = 0;
            foreach (var device in children)
            {
                var node = device.Value<string>("node");
                var localDevice = localState.Devices[i];
                localDevice.DeSerializeConfiguration(device);
                i++;
            }
        }
    }

    void SendDeviceInfo()
    {
        _ = _socket.SendAsync(new
        {
            action = "device info",
            deviceCount = NetState.Devices.Count, // TODO remove this line
            info = (from d in NetState.Devices where d.HasData select d.SerializeConfiguration()).ToArray(),
        });
        NetState.DeviceInfoChanged = false;
    }

    public Isbl.NetStateData NetState = new();
    public Dictionary<int, Isbl.NetStateData> OtherStates = new();

    public void Dispose()
    {
        _socket?.Dispose();
    }

    /// <summary>
    /// Periodically called from IsblNetComponent to upload current NetState to
    /// server via UDP
    /// </summary>
    public void Tick()
    {
        if (!NetState.Initialized || _socket == null) return;
        var bytes = new byte[NetState.CalculateSerializationSize()];
        var span = bytes.AsSpan();
        BinaryPrimitives.WriteInt32LittleEndian(span[0..4], NetState.Id);
        int offset = 4;
        offset += Isbl.NetData.Write7BitEncodedInt(span[offset..], NetState.Devices.Count(d => d.HasData));
        foreach (var device in NetState.Devices)
        {
            if (!device.HasData) continue;
            var len = device.CalculateSerializationSize();
            var realLen = device.SerializeData(bytes, offset);
            if (len != realLen) Debug.LogWarning($"Length mismatch: theoretical {len} vs real {realLen}");
            offset += len;
        }
        if (offset != bytes.Length) Debug.LogWarning($"Wrong final offset: theoretical {bytes.Length} vs real {offset}");

        _ = _socket.SendAsync(bytes, System.Net.WebSockets.WebSocketMessageType.Binary);
        if (NetState.DeviceInfoChanged) SendDeviceInfo();
    }
}
