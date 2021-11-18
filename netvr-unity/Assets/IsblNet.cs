using System;
using Newtonsoft.Json;
using UnityEngine;
using System.Linq;
using System.Collections.Generic;
using Newtonsoft.Json.Linq;
using UnityEngine.XR;

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
    public static void EnsureInstanceExists(bool printDebug) { if (!IsblNetComponent.InstanceExists) IsblNetComponent.Instance = new(printDebug); }

    /// <summary>
    /// When you need to send or recieve message via network use this to access
    /// primary IsblNet instance
    /// </summary>
    public static IsblNet Instance { get { EnsureInstanceExists(false); return IsblNetComponent.Instance; } }

    public readonly ReconnectingClientWebSocket Socket;
    public bool Connected => false;

    /// <summary>Private constructor. Only place where this is called is from
    /// EnsureInstanceExists and that only happens if other instance does not
    /// exist already.</summary>
    IsblNet(bool printDebug)
    {
        Socket = new("ws://192.168.1.31:10000", printDebug);
        Socket.OnConnect += () =>
        {
            if (NetState.Id == 0) Socket.SendAsync(new { action = "gimme id" });
            else Socket.SendAsync(new { action = "i already has id", id = NetState.Id, token = NetState.IdToken });
        };
        Socket.OnDisconnect += () =>
        {
            NetState.Initialized = false;
            OtherStates.Clear();
        };
        Socket.OnTextMessage += (text) =>
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
                    NetState.Initialized = true;
                    NetState.DeviceInfoChanged = true;
                }
                else if (action == "device info")
                {
                    HandleDeviceInfo(obj);
                }
                else
                {
                    Debug.Log($"Unknown action \"{action}\"");
                }
            }
            catch (Exception e)
            {
                Debug.LogError(e);
            }
        };
        Socket.OnBinaryMessage += (data) =>
            {
                int count = BitConverter.ToInt32(data, 0);
                var offset = 4;

                for (int i = 0; i < count; ++i)
                {
                    var id = BitConverter.ToInt32(data, offset);
                    if (OtherStates.ContainsKey(id))
                    {
                        var state = OtherStates[id];
                        Isbl.NetData.Convert(ref offset, state, data, toBinary: false);
                    }
                }
            };
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
                    Left = new(),
                    Right = new(),
                };
                OtherStates.Add(id, localState);
            }
            if (!peer.ContainsKey("info")) continue;

            foreach (var device in peer.GetValue("info").Children<JObject>())
            {
                var node = device.Value<string>("node");
                if (node != "left" && node != "right") continue;
                var localDevice = node == "left" ? localState.Left : localState.Right;
                localDevice.Name = device.Value<string>("name");
                localDevice.Characteristics = (InputDeviceCharacteristics)device.Value<int>("characteristics");
            }
        }
    }

    void SendDeviceInfo()
    {
        _ = Socket.SendAsync(new
        {
            action = "device info",
            info = new[] {
                new { node = "left", name = NetState.Left.Name, characteristics = NetState.Left.Characteristics },
                new { node = "right", name = NetState.Right.Name, characteristics = NetState.Right.Characteristics },
            }
        });
        NetState.DeviceInfoChanged = false;
    }

    public Isbl.NetStateData NetState = new();
    public Dictionary<int, Isbl.NetStateData> OtherStates = new();

    public void Dispose()
    {
        Socket.Dispose();
    }

    /// <summary>
    /// Periodically called from IsblNetComponent to upload current NetState to
    /// server via UDP
    /// </summary>
    public void Tick()
    {
        if (!NetState.Initialized) return;
        var bytes = new byte[Isbl.NetStateData.ByteLength];
        int offset = 0;
        Isbl.NetData.Convert(ref offset, NetState, bytes, true);
        _ = Socket.SendAsync(bytes, System.Net.WebSockets.WebSocketMessageType.Binary);
        if (NetState.DeviceInfoChanged) SendDeviceInfo();
    }
}
