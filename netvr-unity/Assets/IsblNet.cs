using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Text;
using UnityEngine;

public class IsblNet : IDisposable
{
    public static IsblNet Instance { get { return IsblNetComponent.Instance; } }

    public const int ServerPort = 10000;
    readonly UdpClient _client;

    public IsblNet()
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
