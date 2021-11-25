import { Peer } from './peer.js'
import { getRandomString, promisifyWebsocket } from './utils.js'

export function createRoom() {
  let idgen = 0
  const peersById = new Map<number, Peer>()
  const peerTokens = new Map<number, string>()

  return { interval: 15, onSocket }

  /**
   * Function to handle incoming connection from client.
   *
   * @param conn object representing open tcp connection
   */
  async function onSocket(socketIn: WebSocket) {
    const socket = await promisifyWebsocket(socketIn)
    let thisPeer: Peer | null = null
    try {
      for await (const event of socket) {
        if (event.data instanceof ArrayBuffer) {
          if (!thisPeer) {
            console.error('Invoked binary action without setting up id first')
          } else {
            thisPeer.onBinary(
              event.data,
              other(peersById.values(), thisPeer.id),
            )
          }
          continue
        }
        const message = JSON.parse(event.data)
        if (message.action === 'gimme id') {
          thisPeer = createPeer(++idgen, socket)
          socket.send(
            JSON.stringify({
              action: "id's here",
              intValue: thisPeer.id,
              stringValue: peerTokens.get(thisPeer.id),
            }),
          )
        } else if (message.action === 'i already has id') {
          const requestedId = message.id
          const token = peerTokens.get(requestedId)
          if (token && token === message.token) {
            thisPeer = createPeer(requestedId, socket)
            console.log(peersById)
            socket.send(JSON.stringify({ action: 'id ack' }))
          } else {
            thisPeer = createPeer(++idgen, socket)
            console.log(peersById)
            socket.send(
              JSON.stringify({
                action: "id's here",
                intValue: thisPeer.id,
                stringValue: peerTokens.get(thisPeer.id),
              }),
            )
          }
        } else if (thisPeer) {
          thisPeer.onJson(message, other(peersById.values(), thisPeer.id))
        } else if (message.action === 'keep alive') {
          /* ignore */
        } else {
          console.error(
            'Invoked',
            JSON.stringify(message.action),
            'without setting up id first',
          )
        }
      }
    } finally {
      if (thisPeer) {
        peersById.delete(thisPeer.id)
        console.log(peersById)
      }
      console.log('Socket finished')
    }
  }

  function createPeer(...params: ConstructorParameters<typeof Peer>) {
    const peer = new Peer(...params)

    const token = peerTokens.get(peer.id) ?? getRandomString(64)
    peerTokens.set(peer.id, token)

    peersById.set(peer.id, peer)
    console.log(peersById)
    return peer
  }
}

function* otherImpl(peers: Iterable<Peer>, selfId: number) {
  for (const peer of peers) {
    if (peer.id !== selfId) yield peer
  }
}

function other(peers: Iterable<Peer>, selfId: number) {
  return Array.from(otherImpl(peers, selfId))
}
