using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Text;
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
    readonly UdpClient _client;

    /// <summary>Private constructor. Only place where this is called is from
    /// EnsureInstanceExists and that only happens if other instance does not
    /// exist already.</summary>
    IsblNet()
    {
        _client = new UdpClient("127.0.0.1", ServerPort);
        UDPTest();
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
    }
}
