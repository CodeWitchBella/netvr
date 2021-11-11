using System;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;
using UnityEngine;

/// <summary>
/// Encapsulates TcpClient with reconnect logic.
/// </summary>
///
/// It handles:
/// - parsing incoming stream into separate messages and .Invokes OnMessage,
/// - cancels everything when disposed
/// - queues messages until TCP connection is established
///
/// Note that there is a possibility that last message in flight might get
/// dropped (see TODO below). It's possible to fix, but I don't want to bother
/// right now.
class ReconnectingTcpClient : IDisposable
{
    TcpClient _client;
    Task _task;
    CancellationTokenSource _cancellationToken;
    readonly string _host;
    readonly int _port;

    public bool Connected => _client?.Connected ?? false;

    public delegate void MessageHandler(string message);
    public MessageHandler OnMessage;

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
            _cancellationToken?.Cancel();
            _cancellationToken = new();
            var client = new TcpClient();
            try { await client.ConnectAsync(_host, _port); }
            catch (Exception e)
            {
                Debug.LogWarning(e);
                timeout *= 2;
                if (timeout > 60_000) timeout = 60_000;
                await Task.Delay(timeout);
            }

            if (client.Connected)
            {
                _client = client;
                StartReading(_cancellationToken.Token);
            }
        }
    }

    /// <summary>
    /// Reads bytes from client in a loop and generates messages as OnMessage events.
    /// </summary>
    async void StartReading(CancellationToken token)
    {
        try
        {
            var buffer = new byte[2048];
            var remainingBytes = new byte[0];
            while (_client.Connected)
            {
                var bytes = await _client.GetStream().ReadAsync(buffer, token);
                remainingBytes = ConcatByteArrays(remainingBytes, buffer.AsSpan(0, bytes));

                while (remainingBytes.Length > 4)
                {
                    var length = ReadLength(remainingBytes);
                    if (remainingBytes.Length < length + 4) break;

                    var message = System.Text.Encoding.UTF8.GetString(remainingBytes.AsSpan(4, length));
                    try { OnMessage?.Invoke(message); }
                    catch (Exception e) { Debug.LogError(e); }

                    // remove message bytes
                    var newRemainingBytes = new byte[remainingBytes.Length - length - 4];
                    remainingBytes.AsSpan(length + 4).CopyTo(newRemainingBytes);
                    remainingBytes = newRemainingBytes;
                }
            }
        }
        catch (TaskCanceledException) { /* ignore */ }
        catch (Exception e) { if (Application.isPlaying) Debug.LogError(e); }
    }

    /// <summary>
    /// Reads four bytes as little-endian uint32
    /// </summary>
    int ReadLength(byte[] bytes)
    {
#pragma warning disable RCS1123 // extra parenthesis would not make this easier to read...
        return bytes[0] + 256 * (bytes[1] + 256 * (bytes[2] + 256 * bytes[3]));
#pragma warning restore RCS1123
    }

    /// <summary>
    /// Merges two Span&lt;byte&gt; together to create byte[]
    /// </summary>
    static byte[] ConcatByteArrays(Span<byte> a, Span<byte> b)
    {
        var newBytes = new byte[a.Length + b.Length];
        a.CopyTo(newBytes);
        b.CopyTo(newBytes.AsSpan(a.Length));
        return newBytes;
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
        catch (TaskCanceledException) { return; }
        catch (Exception e) { Debug.LogError(e); }

        if (_cancellationToken.IsCancellationRequested) return;
        try { await EnsureConnected(); }
        catch (TaskCanceledException) { return; }
        if (_cancellationToken.IsCancellationRequested) return;

        try { await _client.GetStream().WriteAsync(bytes, _cancellationToken.Token); }
        catch (TaskCanceledException) { return; }
        catch (Exception e)
        {
            // TODO: if this happens, message is gonna be dropped...
            // I should requeue it again here (but in the front of the queue)
            // It would probably be better to do explicit while(true) loop than
            // this callback magic dance, and use a actual queue of byte[]s
            Debug.LogError(e);
        }
    }

    public void Dispose()
    {
        _cancellationToken?.Cancel();
        _client?.Dispose();
    }
}
