type Await<T extends Promise<unknown>> = T extends Promise<infer V> ? V : never;
export type PWebSocket = Await<ReturnType<typeof promisifyWebsocket>>;

export async function promisifyWebsocket<Message = any>(
  socket: WebSocket,
) {
  const messageQueue: ({ type: "message"; value: MessageEvent<Message> } | {
    type: "error";
    value: unknown;
  } | { type: "close" })[] = [];
  let finished = false;

  let onQueueHasMessage: (() => void) | null = null;

  socket.onmessage = (e) => {
    messageQueue.push({ type: "message", value: e });
    onQueueHasMessage?.();
  };
  socket.onerror = (e) => {
    messageQueue.push({ type: "error", value: e });
    onQueueHasMessage?.();
    finished = true;
  };
  socket.onclose = () => {
    messageQueue.push({ type: "close" });
    onQueueHasMessage?.();
    finished = true;
  };

  await new Promise<void>((resolve) => {
    socket.onopen = () => resolve();
    onQueueHasMessage = resolve;
  });
  onQueueHasMessage = null;

  return {
    get bufferedAmount() {
      return socket.bufferedAmount;
    },
    send(data: string | ArrayBufferLike | Blob | ArrayBufferView): void {
      console.log(
        "Sending",
        typeof data === "string" ? `(${data.length})` : "",
        data,
      );
      socket.send(data);
    },
    [Symbol.asyncIterator](): AsyncIterator<MessageEvent<Message>> {
      return {
        next() {
          const promise = new Promise<
            IteratorResult<MessageEvent<Message>>
          >(
            (resolve, reject) => {
              onQueueHasMessage = () => {
                onQueueHasMessage = null;
                const message = messageQueue.splice(0, 1)[0];
                if (message.type === "message") {
                  resolve({ value: message.value });
                } else if (message.type === "error") {
                  reject(message.value);
                } else {
                  resolve({ done: true, value: null });
                }
              };
            },
          );
          if (messageQueue.length > 0) {
            // run this even if finished to properly consume queue first
            onQueueHasMessage?.();
          } else if (finished) {
            // if next is called after resolve with { done: true } error out
            return Promise.reject(new Error("WebSocket is closed"));
          }
          return promise;
        },
      };
    },
  };
}

export function getRandomString(s: number) {
  if (s % 2 == 1) {
    throw new Deno.errors.InvalidData("Only even sizes are supported");
  }
  const buf = new Uint8Array(s / 2);
  crypto.getRandomValues(buf);
  let ret = "";
  for (let i = 0; i < buf.length; ++i) {
    ret += ("0" + buf[i].toString(16)).slice(-2);
  }
  return ret;
}
