# Technický design

Dále bylo potřeba rozhodnout, jak zařídit vykreslování ovladačů. Jelikož plánuji
podporovat síťový multiplayer s použitím různých headsetů ve stejném sezení
musím být schopná načíst modely ovladačů ostatních headsetů. Jedna cesta, kterou
se ubírá mnoho VR titulů je nepoužít realistické modely reálných headsetů, ale
jiné zástupné modely. Já jsem se z důvodu lepší uživatelské přívětivosti a
snazšího vysvětlování novým uživatelům rozhodla ovladače načítat. Jako zdroj
modelů jsem použila WebXR Input Profiles projektu Immersive Web:cite[controller-models]. Ovladače
načítám pomocí knihovny glTFast z OpenUPM:cite[openupm]. Zde se ukázalo, že modely jsou
chybně umístěné relativně vůči realitě. Toto znamená, že mohu podporovat
pouze headsety ve kterých mojí aplikaci můžu vyzkoušet, což je trochu
nepříjemné. Uvažuji nad vytvoření modelů z .obj souborů dostupných v instalaci
SteamVR, které se zdají být přesnější. Prozatím to však stačí a modely pro
Oculus Touch controllery a Vive Wands mám zarovnané správně.

Pro načítání dalších modelů headsetů bude potřeba obejít unity, protože
neposkytuje dostatečně detailní popis zařízení - pouze nespecifický Head
Tracking - OpenXR. Pro nalezení informací, které OpenXR poskytuje, ale unity ne
lze použít OpenXR Explorer:cite[openxr-explorer]

Pro umístění modelů ve virtuálním světě a reakci na stisknutí jejich tlačítek je
ještě nutné vybrat input systém - unity legacy, nebo nový input systém. Zde jsem
se rozhodla nepoužít ani jeden a místo nich použít třídy z namespacu
UnityEngine.XR, neboť mi umožňují přímo přečíst stav ovladačů v mnou stanoveném
momentě bez přílišných ceremonií.

Čtení vstupu z XR.InputDevice jsem zabalila do jedné třídy IsblXRDevice, která
obsahuje getter pro každé tlačítko dostupné na Vive Wands, nebo na ovladačích
Oculus Touch. Pro tlačítka, která se jinak jmenují, ale mají stejnou funkci
(Menu a MenuButton) obsahuje jenom jeden getter a pro chybějící tlačítka
generuje rozumnou fallback hodnotu, nebo vyhodí výjimku. Z této třídy čte třída
IsblStaticXRDevice, která přímo v sobě ukládá stav tlačítek - toto je pro přenos
po síti. Všechny interakce s ovladači probíhají přes IsblStaticXRDevice, což
znamená, že fungují nezávisle na tom, jestli jsou lokální nebo vzdálené.

## Online část

Pro komunikaci mezi klienty jsem se rozhodla...

<!--
TODO, WebSockets (works everywhere, possible to encrypt, well defined, but TCP
with head-of-line blocking), deno (because compile), miniflare, cloudflare
workers (because edge). UDP/TCP possible future, but seems to work.
Velocity/angular velocity delta time extrapolation. Same frame updates. Where to
transform? Server or client-side.

The system has to transform not only the position and rotation of all connected
devices but also their velocity and angular velocity.
-->

## Communication protocol

**Configuration**

JSON, needs reliable transport. Only sent on change, received upon change or new
connection. Contains information about layout of data messages.

<!--
TODO: describe json messages once I feel like I won’t be changing them.
-->

**action: calibration**

calibrations[]: ... Must never arrive before the “device info” message.

**Uploading data to server**

Binary message. WebSockets do have flag for that, so there is no need to
distinguish them and in the UDP future there will be fully separate connection
there, so no need to distinguish those either.

Everything is little-endian (C# implementation note: use System.Buffers.Binary
.BinaryPrimitives instead of System.BitConverter to make sure endianness is
correct).

Some integers are written as 7BitEncodedInt which is same as in C#’s binary
writer:cite[7bit-int]. It makes sure that smaller integers are encoded as fewer bytes with
the tradeoff that large integers (>= 2^28) take five bytes.

Int32 Client ID. It is associated with the connection which means that it does
not need to be there, but UDP does not have any concept of connections so I’ll
put it there anyway. Also makes the encoding more symmetrical.

<!--Port remaining docs, figure out formatting-->

**Receiving data from server**

Each message can contain data about multiple clients. If data about a client is
not present it means that data did not change (not a disconnect).

Int32 Number of clients For each client: format same as messages sent to server

## Device encoding

<!--Port from google docs, figure out formatting-->
