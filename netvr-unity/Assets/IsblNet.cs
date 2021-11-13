using System;
using Newtonsoft.Json;
using UnityEngine;
using System.Linq;

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
        Socket = new("ws://127.0.0.1:10000", printDebug);
        Socket.OnConnect += () =>
        {
            if (NetState.Id == 0) Socket.SendAsync(new { action = "gimme id" });
            else Socket.SendAsync(new { action = "i already has id", id = NetState.Id, token = NetState.IdToken });
        };
        Socket.OnTextMessage += (text) =>
        {
            var obj = JsonConvert.DeserializeObject<Isbl.NetIncomingTCPMessage>(text);
            if (obj.Action == "id's here")
            {
                NetState.Id = obj.IntValue;
                NetState.IdToken = obj.StringValue;
            }
            Debug.Log(text);
        };
        Socket.OnBinaryMessage += (data) =>
        {
            var text = from i in data select i.ToString();
            Debug.Log($"binary message ({data.Length}) {string.Join(" ", text)}");
        };
    }

    public Isbl.NetStateData NetState;

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
        if (NetState.Id < 1) return;
        var bytes = new byte[Isbl.NetStateData.ByteLength];
        Isbl.NetData.WriteTo(NetState, bytes, 0);
        _ = Socket.SendAsync(bytes, System.Net.WebSockets.WebSocketMessageType.Binary);
    }
}
