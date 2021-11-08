/**
 * Simple deno script used to test some basic server functionality without
 * having to run the server.
 *
 * Example invocation:
 * $ deno run --unstable --allow-net test.js
 */

const l = listenRandomPort();
console.log(l.addr);
await l.send(new TextEncoder().encode("abcd"), {
  transport: "udp",
  hostname: "127.0.0.1",
  port: 10000,
});

/**
 * Generates random port in range 1001-65000, tries to UDP listen to and if it
 * succeedes then it returns port. If it fails then it retries up to 100 times.
 */
function listenRandomPort() {
  const limit = 100;
  for (let i = 0; ; ++i) {
    try {
      return Deno.listenDatagram({
        port: 1001 + Math.floor(Math.random() * 64000),
        transport: "udp",
      });
    } catch (e) {
      if (e.name !== "AddrInUse" || i >= limit) throw e;
    }
  }
}
