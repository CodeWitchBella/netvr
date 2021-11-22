import { promisifyWebsocket, PWebSocket } from "./utils";

export async function run() {
  const events = document.querySelector("#events")!;
  if (!events) throw new Error("Cant find #events");
  const socket = await promisifyWebsocket(createSocket());
  send(socket, { action: "gimme id" });
  setInterval(() => {
    socket.send(JSON.stringify({ action: "keepalive" }));
  }, 250);
  for await (const message of socket) {
    if (
      typeof message.data === "string" &&
      JSON.parse(message.data)["action"] === "keep alive"
    ) {
      continue;
    }
    const div = document.createElement("div");
    events.appendChild(div);
    clear();
    div.className = "event";
    div.innerHTML = `
      <div>🔽 ${new Date().toISOString()}</div>
      ${
      typeof message.data === "string"
        ? "<pre>" +
          JSON.stringify(JSON.parse(message.data), null, 2) +
          "</pre>"
        : stringify(message.data)
    }
      `;
  }

  function send(socket: PWebSocket, data: unknown) {
    const div = document.createElement("div");
    events.appendChild(div);
    clear();
    div.className = "event";
    div.innerHTML = `
    <div>🔼 ${new Date().toISOString()}</div>
    <pre>${JSON.stringify(data, null, 2)}</pre>
    `;
    socket.send(JSON.stringify(data));
  }

  function clear() {
    while (events.childElementCount > 10) {
      events.removeChild(events.children[0]);
    }
  }

  function stringify(data: ArrayBuffer) {
    let view = new DataView(data, 0, data.byteLength);
    const length = view.getInt32(0, true);
    let res = "<div>";
    for (let i = 0; i < length; ++i) {
      view = new DataView(data, 4 + 79 * i, 79);
      res += `#${view.getInt32(0, true)}`;
      res += getTypePosRot(view, 4);
      res += getTypePosRot(view, 29);
      res += getTypePosRot(view, 54);
    }
    res += "</div>";
    return res;
  }

  function getTypePosRot(view: DataView, offset: number) {
    return `<div class="device">
      type: ${view.getUint8(offset)}
      position: ${getVector3(view, offset + 1)}
      rotation: ${getVector3(view, offset + 13)}</div>`;
  }

  function getVector3(view: DataView, offset: number) {
    const fixed = 3;
    return `${view.getFloat32(offset, true).toFixed(fixed)}, ${
      view
        .getFloat32(offset + 4, true)
        .toFixed(fixed)
    }, ${
      view
        .getFloat32(offset + 8, true)
        .toFixed(fixed)
    }`;
  }

  function createSocket() {
    const socketUrl = new URL(window.location.toString());
    socketUrl.pathname = "/ws";
    socketUrl.protocol = socketUrl.protocol === "https:" ? "wss:" : "ws:";
    if (socketUrl.port === "3000") socketUrl.port = "10000";
    const socket = new WebSocket(socketUrl.toString());
    socket.binaryType = "arraybuffer";
    return socket;
  }
}
