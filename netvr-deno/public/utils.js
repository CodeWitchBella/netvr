// detypescriptified version of utils.ts for direct use in browser

export async function promisifyWebsocket(socket) {
  const messageQueue = [];
  let finished = false;

  let onQueueHasMessage = null;

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

  await new Promise((resolve) => {
    socket.onopen = () => resolve();
    onQueueHasMessage = resolve;
  });
  onQueueHasMessage = null;

  return {
    get bufferedAmount() {
      const amount = socket.bufferedAmount;
      return Number.isNaN(amount) ? 0 : amount;
    },
    send(data) {
      socket.send(data);
    },
    [Symbol.asyncIterator]() {
      return {
        next() {
          const promise = new Promise((resolve, reject) => {
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
          });
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
