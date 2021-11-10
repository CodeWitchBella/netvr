using System;
using System.Net.Sockets;
using System.Text;
using Newtonsoft.Json;
using UnityEngine;

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

    public const int ServerPort = 10000;
    public const string ServerHost = "127.0.0.1";
    readonly UdpClient _client;
    readonly ReconnectingTcpClient _tcp;
    public bool TcpConnected => _tcp.Connected;

    /// <summary>Private constructor. Only place where this is called is from
    /// EnsureInstanceExists and that only happens if other instance does not
    /// exist already.</summary>
    IsblNet()
    {
        _client = new UdpClient(ServerHost, ServerPort);
        _tcp = new ReconnectingTcpClient(ServerHost, ServerPort);

        UDPTest();
        TCPSend(new { type = "hello" });
        TCPSend(new { type = "hello world" });
    }

    /// <summary>
    /// Sends JSON-formatted message via reliable connection of TCP channel.
    /// Do not use for time-critical updates.
    /// </summary>
    public void TCPSend(object data)
    {
        // the 1234 pre-creates space for 4byte size information
        // this skips extra copy and avoids sending packet just with size
        // could probably be done better but this works ¯\_(ツ)_/¯ 🤷‍♀️
        var jsonBytes = Encoding.UTF8.GetBytes("1234" + JsonConvert.SerializeObject(data) + "\n");
        BitConverter.TryWriteBytes(jsonBytes, jsonBytes.Length - 4);
        _tcp.QueueBytes(jsonBytes);
    }

    async void UDPTest()
    {
        //_client.Connect("192.168.1.31", 10000);
        byte[] sendBytes = Encoding.ASCII.GetBytes("Hello, from the client");
        await _client.SendAsync(sendBytes, sendBytes.Length);

        UdpReceiveResult recv = await _client.ReceiveAsync();
        string receivedString = Encoding.ASCII.GetString(recv.Buffer);
        Debug.Log("Message received from the server \n " + receivedString);
    }

    public void Dispose()
    {
        _client.Dispose();
        _tcp.Dispose();
    }
}
