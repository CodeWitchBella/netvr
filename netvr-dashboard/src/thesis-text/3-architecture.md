# Application Architecture {#architecture}

:todo[Describe how the application is implemented]

gist: Client-server architecture with dedicated server, which is deployed either as standalone .exe file on LAN, or as Cloudflare Worker for lowest possible latency. WebSockets, not UDP - adds latency, but that is handled.

## Native API Choice {#native-api-choice}

Many headsets have their own API - for example VRAPI on Oculus/Meta headsets, OpenVR on headsets which use SteamVR (like HTC VIVE). More recently headsets started supporting unified API called OpenXR, which solves a lot of problems with portability, but being newer API it has few missing parts. :todo[this is unfinished]
