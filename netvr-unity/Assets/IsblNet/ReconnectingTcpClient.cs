using System;
using System.Net.Sockets;
using System.Threading.Tasks;
using UnityEngine;
/// <summary>
/// Encapsulates TcpClient with reconnect logic
/// </summary>
class ReconnectingTcpClient : IDisposable
{
    TcpClient _client;
    Task _task;
    readonly Task _pollTask;
    readonly string _host;
    readonly int _port;

    public bool Connected => _client?.Connected ?? false;

    public ReconnectingTcpClient(string host, int port)
    {
        _host = host;
        _port = port;
        _task = EnsureConnected();
    }

    /// <summary>
    /// Connects to server via TCP with exponential backoff. Noop if already connected
    /// </summary>
    async Task EnsureConnected()
    {
        if (_client?.Connected == false) { _client.Dispose(); _client = null; }
        int timeout = 50;
        while (_client == null)
        {
            var client = new TcpClient();
            try
            {
                await client.ConnectAsync(_host, _port);
            }
            catch (Exception e)
            {
                Debug.LogWarning(e);
                timeout *= 2;
                if (timeout > 60_000) timeout = 60_000;
                await Task.Delay(timeout);
            }
            if (client.Connected) _client = client;
        }
    }

    /// <summary>
    /// Makes sure that client is connected and once that is fullfiled then
    /// sends the data.
    /// </summary>
    public void QueueBytes(byte[] bytes)
    {
        _task = QueueBytesImpl(_task, bytes);
    }

    /// <summary>
    /// Implementation of QueueBytes - first awaits previous task and then sends
    /// data. Returns resulting Task to be awaited before next sending.
    /// </summary>
    async Task QueueBytesImpl(Task prev, byte[] bytes)
    {
        try { await prev; }
        catch (Exception e) { Debug.LogError(e); }

        await EnsureConnected();
        try
        {
            await _client.GetStream().WriteAsync(bytes);
        }
        catch (Exception e)
        {
            Debug.LogError(e);
        }
    }

    public void Dispose()
    {
        _client.Dispose();
        _task.Dispose();
        _pollTask.Dispose();
    }
}
