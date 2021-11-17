/**
 * Server used for relaying messages between multiple users. Alternative
 * entrypoint intended for running on cloudflare workers.
 *
 * Example invocation:
 * $ yarn dev
 */

import { getAssetFromKV } from "@cloudflare/kv-asset-handler";
import manifestJSON from "__STATIC_CONTENT_MANIFEST";
import { createRoom } from "./room";
const manifest = JSON.parse(manifestJSON);

export default {
  async fetch(request: Request, env: any, ctx: ExecutionContext) {
    const upgradeHeader = request.headers.get("Upgrade");
    if (upgradeHeader === "websocket") {
      const id = env.WEBSOCKET.idFromName("test");
      const object = env.WEBSOCKET.get(id);
      return object.fetch(request.url, request);
    }
    try {
      return await getAssetFromKV(
        {
          request,
          waitUntil: (promise: any) => ctx.waitUntil(promise),
        } as any,
        { ASSET_NAMESPACE: env.__STATIC_CONTENT, ASSET_MANIFEST: manifest },
      );
    } catch (e: any) {
      if (e.status === 404) return new Response("Not found", { status: 404 });
      return new Response(e.message, { status: e.status });
    }
  },
};

export class DurableObjectWebSocket {
  state;
  room = createRoom();
  constructor(state: any, env: any) {
    this.state = state;
  }

  async fetch(request: Request) {
    const ip = request.headers.get("CF-Connecting-IP");

    const [client, server] = Object.values(new WebSocketPair());

    this.room.onSocket(server).catch((e) => {
      console.error(e);
    });
    (server as any).onopen(); // workaround missing onopen call
    server.accept();
    server.send(JSON.stringify({ ip }));

    return new Response(null, {
      status: 101,
      webSocket: client,
    });

    let data = await request.text();
    let storagePromise = this.state.storage.put(ip, data);
    await storagePromise;
    return new Response(ip + " stored " + data);
  }
}
