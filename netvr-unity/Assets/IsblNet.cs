using System;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Text;
using UnityEngine;

public class IsblNet : MonoBehaviour
{
    #region singleton
    static IsblNet _instance;
    public static IsblNet Instance
    {
        get
        {
            if (_instance != null) return _instance;
            _instance = FindObjectOfType<IsblNet>();
            if (_instance != null) return _instance;
            GameObject go = new("IsblNet");
            _instance = go.AddComponent<IsblNet>();
            return _instance;
        }
    }

    void OnDestroy()
    {
        if (_instance == this) _instance = null;
    }
    #endregion singleton


    void Start()
    {
        UDPTest();
    }
    async void UDPTest()
    {
        UdpClient client = new UdpClient(5600);
        try
        {
            //client.Connect("192.168.1.31", 10000);
            client.Connect("127.0.0.1", 10000);
            byte[] sendBytes = Encoding.ASCII.GetBytes("Hello, from the client");
            await client.SendAsync(sendBytes, sendBytes.Length);

            UdpReceiveResult recv = await client.ReceiveAsync();
            string receivedString = Encoding.ASCII.GetString(recv.Buffer);
            Debug.Log("Message received from the server \n " + receivedString);
        }
        catch (Exception e)
        {
            Debug.Log("Exception thrown " + e.Message);
        }
    }
}
