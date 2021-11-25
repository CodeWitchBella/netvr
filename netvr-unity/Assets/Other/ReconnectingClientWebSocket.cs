using System;
using System.Net.WebSockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using UnityEngine;
using Newtonsoft.Json;
using System.Linq;

public class ReconnectingClientWebSocket : IDisposable
{
    readonly CancellationTokenSource _cancellationTokenSource = new();
    CancellationTokenSource _keepAliveTokenSource;
    ClientWebSocket _webSocket;
    bool _connecting;
    bool _receiving;
    readonly public Uri Uri;

    public DateTime LastSuccessfulMessage { get; private set; }
    public WebSocketState State => _connecting ? WebSocketState.Connecting : _webSocket?.State ?? WebSocketState.None;

    public Action<string> OnTextMessage;
    public Action<byte[]> OnBinaryMessage;
    public Action OnConnect;
    public Action OnDisconnect;

    public bool PrintDebug;

    public ReconnectingClientWebSocket(string uri)
    {
        Uri = new(uri);
        _ = Connect(fromConstructor: true);
    }

    public async Task SendAsync(byte[] buffer, WebSocketMessageType messageType = WebSocketMessageType.Binary)
    {
        try
        {
            _ = KeepAlive(); // Reset keepalive timer
            await _webSocket.SendAsync(buffer, messageType, true, _cancellationTokenSource.Token);
            LastSuccessfulMessage = DateTime.Now;
        }
        catch (InvalidOperationException)
        {
            _ = Connect();
            throw;
        }
    }
    public Task SendAsync(object obj)
    {
        var text = JsonConvert.SerializeObject(obj);
        // Debug.Log($"Sending: {text}");
        return SendAsync(Encoding.UTF8.GetBytes(text), WebSocketMessageType.Text);
    }

    async Task KeepAlive()
    {
        // cancel previous KeepAlive request which causes the timer to reset
        _keepAliveTokenSource?.Cancel();
        _keepAliveTokenSource = null;
        if (_webSocket.State != WebSocketState.Open) return;
        _keepAliveTokenSource = new();
        var tokenSource = _keepAliveTokenSource;
        var token = tokenSource.Token;
        var message = Encoding.UTF8.GetBytes(@"{""action"":""keep alive""}");
        try
        {
            while (true)
            {
                await Task.Delay(1_000, token);
                if (token.IsCancellationRequested)
                {
                    if (PrintDebug) Debug.Log("Keep alive return 1");
                    return;
                }
                try
                {
                    if (_webSocket?.State == WebSocketState.Open)
                    {
                        if (PrintDebug) Debug.Log("Sending keep alive");
                        await _webSocket.SendAsync(message, WebSocketMessageType.Text, true, token);
                        LastSuccessfulMessage = DateTime.Now;
                    }
                }
                catch (OperationCanceledException)
                {
                    if (PrintDebug) Debug.Log("Keep alive return 2");
                    // keepalive cancelled
                    return;
                }
                catch (WebSocketException)
                {
                    _webSocket = null;
                    /* Keepalive send failed, lets reconnect */
                    _ = Connect();
                }
            }
        }
        catch (OperationCanceledException) { /* ignore expected */ }
        catch (Exception e) { Debug.LogError(e); }
        finally
        {
            tokenSource.Dispose();
            if (_keepAliveTokenSource == tokenSource) _keepAliveTokenSource = null;
        }
    }

    async Task Connect(bool fromConstructor = false)
    {
        if (_connecting || !ShouldReconnect(_webSocket)) return;
        if (PrintDebug) Debug.Log("Initializing connection process");
        try { if (!fromConstructor) OnDisconnect?.Invoke(); } catch (Exception e) { Debug.LogError(e); }

        _connecting = true;
        var timeout = 200;
        try
        {
            do
            {
                _ = CloseNicely(_webSocket);
                _webSocket = new();
                try
                {
                    if (PrintDebug) Debug.Log("Trying to connect");
                    await _webSocket.ConnectAsync(Uri, _cancellationTokenSource.Token);
                }
                catch (OperationCanceledException) { throw; }
                catch (Exception)
                {
                    if (timeout > 60_000) timeout = 60_000;
                    await Task.Delay(timeout, _cancellationTokenSource.Token);
                    timeout *= 2;
                }
            }
            while (_webSocket.State != WebSocketState.Open);
            if (PrintDebug) Debug.Log("Connection success");
            OnConnect?.Invoke();
            _ = StartReceiving();
            _ = KeepAlive();
        }
        catch (OperationCanceledException) { /* ignore, is expected */ }
        catch (Exception e) { Debug.LogError(e); }
        finally
        {
            _connecting = false;
            if (PrintDebug) Debug.Log("Done with connecting");
        }
    }

    async Task StartReceiving()
    {
        if (_receiving) return;
        _receiving = true;
        if (PrintDebug) Debug.Log("Starting receiving");
        var socket = _webSocket;
        var token = _cancellationTokenSource.Token;
        int idx = 0;
        var buffer = new byte[8192];

        try
        {
            while (_webSocket?.State == WebSocketState.Open && !token.IsCancellationRequested)
            {
                var result = await socket.ReceiveAsync(new ArraySegment<byte>(buffer, idx, buffer.Length - idx), token);
                idx += result.Count;
                if (result.EndOfMessage)
                {
                    if (result.MessageType == WebSocketMessageType.Text)
                        OnTextMessage?.Invoke(Encoding.UTF8.GetString(buffer[..idx]));
                    else
                        OnBinaryMessage?.Invoke(buffer[..idx]);
                    idx = 0;
                }
                else if (buffer.Length <= idx + 1024)
                {
                    var newBuffer = new byte[buffer.Length * 2];
                    buffer[..idx].CopyTo(newBuffer, 0);
                    buffer = newBuffer;
                }
            }
        }
        catch (WebSocketException)
        {
            if (PrintDebug) Debug.Log("Connection failed, reconnecting");
            _ = Connect();
        }
        catch (Exception e) { Debug.LogError(e); }
        finally
        {
            _receiving = false;
            if (_webSocket?.State == WebSocketState.Open) _ = StartReceiving();
        }
    }

    static bool ShouldReconnect(ClientWebSocket socket)
    => socket?.State switch
    {
        WebSocketState.Connecting => false,
        WebSocketState.Open => false,
        _ => true,
    };

    async static Task CloseNicely(ClientWebSocket socket)
    {
        if (socket == null) return;
        try
        {
            if (socket.State == WebSocketState.Open)
                await socket.CloseAsync(WebSocketCloseStatus.Empty, "", new());
        }
        catch { /* ignore all errors here */ }

        socket.Dispose();
    }

    public void Dispose()
    {
        OnTextMessage = null;
        OnBinaryMessage = null;
        OnConnect = null;
        OnDisconnect = null;

        _cancellationTokenSource.Cancel();
        _keepAliveTokenSource?.Cancel();
        _ = CloseNicely(_webSocket);
    }

    public void SimulateDisconnect()
    {
        _ = CloseNicely(_webSocket);
    }
}
