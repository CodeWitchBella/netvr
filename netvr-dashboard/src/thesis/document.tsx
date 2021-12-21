import pdf from '@react-pdf/renderer'
import { Page, usePDFContext, TechnikaText } from './base'
import { Chapter, TODO, Paragraph, Section, ParagraphTitle } from './design'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'
const { Document: PDFDocument, StyleSheet, View } = pdf

export function Document() {
  registerFonts()
  const { lang } = usePDFContext()
  return (
    <PDFDocument>
      <TitlePage
        title={
          lang === 'en'
            ? 'Tracking multiple VR users in a shared physical space'
            : 'Sledování více uživatelů VR světa ve sdíleném fyzickém prostoru'
        }
      />
      <Page style={{ alignItems: 'center', justifyContent: 'flex-end' }}>
        <LMText fontFamily="lmroman10-regular">
          Page intentionally left blank
        </LMText>
      </Page>

      <Chapter title="Technický design" no={1}>
        <TODO>Split out as "requirement analysis" and "technical design"</TODO>
        <TODO>Translate to english</TODO>

        <Paragraph first>
          Technický design má mnoho moving parts - interakce s HW, vykreslování
          a načítání modelů, protokol v síťové komunikaci, síťová architektura,
          architektura serveru. Jdou rozdělit na dvě části - offline a online.
        </Paragraph>
        <Section title="Podporované platformy" no={1}>
          <Paragraph first>
            Některá technická rozhodnutí jsou přímo ovlivněna cílovými
            zařízeními a platformami na kterých jsou tato zařízení podporována.
            Pro dosažení výsledné funkcionality je potřeba, aby cílové zařízení
            podporovalo poziční, room-scale trackování.
          </Paragraph>
          <Paragraph>
            Rozhodla jsem se prvotní prototyp implementovat pro zástupce dvou
            způsobů sledování pohybu headsetu. Outside-in a inside-out.
          </Paragraph>
          <Paragraph>
            Jako zástupce outside-in jsem si zvolila HTC Vive jakožto zástupce
            sledovací technologie Lighthouse. Druhý běžný outside-in sledovací
            systém je systém Constellation použitý na headsetech Oculus Rift,
            který ale nemá v současnosti prodávaného zástupce.
          </Paragraph>
          <Paragraph>
            Jako zástupce inside-out jsem zvolila Oculus Quest 2 používající
            sledovací systém Insight. Toto je nejčastější headset používaný i v
            desktop módu podle Steam Hardware Survey[4]. Aplikace je na tomto
            headsetu možné spouštět dvěmi způsoby - přímo na headsetu nebo na
            připojeném počítači. Chtěla bych podporovat obě možnosti. Na
            připojení počítače je možné použít několik různých kombinací
            software. Jedna možnost je připojení přes USB a použití Oculus Link
            s nebo bez SteamVR. Další je použití WiFi a Oculus AirLink, opět s
            použitím SteamVR, nebo bez použití SteamVR. Existuje i několik
            dalších možností jako je AMD ReliveVR[6], Virtual Desktop (ten jsem
            nezkoušela, neboť je placený a vypadá to, že mám problém s
            enkódování videostreamu na počítači s čímž by VD nepomohl) nebo
            ALVR[5], které všechny fungují přes SteamVR a WiFi. Zde mi na mém
            počítači bez znatelné latence fungoval pouze Oculus Link přes USB,
            takže jsem se rozhodla použít ten v kombinaci se SteamVR pro
            snadnější přepínání mezi headsety při testování. (holy mother of
            condensed text, tohle asi pro výslednou práci smažu, ale mám to
            tady, abych věděla co jsem zkoušela)
          </Paragraph>
        </Section>
        <Section title="Offline část" no={2}>
          <Paragraph first>
            Pro implementaci jsem se rozhodla použít engine Unity[2] a to hlavně
            z toho důvodu, že s ním umím a také protože při pokusu o použití
            Unreal Engine 5 preview 2[3] mi Unreal vykresloval špatný obraz do
            pravého oka a mě se to nechtělo řešit.
          </Paragraph>

          <Paragraph>
            Unity poskytuje dvě metody vykreslování VR obsahu. Platform specific
            API (OpenVR při použití SteamVR, VRAPI na Oculus platformách), nebo
            Khronos OpenXR. Platform specific API jsou starší a v některých
            případech poskytují víc funkcionality. Oproti tomu OpenXR je
            technologie, která v budoucnu má všude staré API nahradit (například
            na platformě Meta Quest je VRAPI deprecated). Hlavní nevýhoda OpenXR
            je, že mimo Microsoft-only rozšíření neobsahuje způsob jak získat
            modely ovladačů a nedostupnost plně funkčního hand trackingu na Meta
            Questu. Zde jsem se rozhodla použít OpenXR, protože s jeho použitím
            není třeba nijak upravovat kód pro spuštění na Meta Quest 2 vs HTC
            Vive.
          </Paragraph>
          <Paragraph>
            Dále bylo potřeba rozhodnout, jak zařídit vykreslování ovladačů.
            Jelikož plánuji podporovat síťový multiplayer s použitím různých
            headsetů ve stejném sezení musím být schopná načíst modely ovladačů
            ostatních headsetů. Jedna cesta, kterou se ubírá mnoho VR titulů je
            nepoužít realistické modely reálných headsetů, ale jiné zástupné
            modely. Já jsem se z důvodu lepší uživatelské přívětivosti a
            snazšího vysvětlování novým uživatelům rozhodla ovladače načítat.
            Jako zdroj modelů jsem použila WebXR Input Profiles projektu
            Immersive Web[7]. Ovladače načítám pomocí knihovny glTFast[9] z
            OpenUPM[10]. Zde se ukázalo, že modely jsou chybně umístěné
            relativně vůči realitě[8]. Toto znamená, že mohu podporovat pouze
            headsety ve kterých mojí aplikaci můžu vyzkoušet, což je trochu
            nepříjemné. Uvažuji nad vytvoření modelů z .obj souborů dostupných v
            instalaci SteamVR, které se zdají být přesnější. Prozatím to však
            stačí a modely pro Oculus Touch controllery a Vive Wands mám
            zarovnané správně.
          </Paragraph>
          <Paragraph>
            Pro načítání dalších modelů headsetů bude potřeba obejít unity,
            protože neposkytuje dostatečně detailní popis zařízení - pouze
            nespecifický Head Tracking - OpenXR. Pro nalezení informací, které
            OpenXR poskytuje, ale unity ne lze použít OpenXR Explorer[14]
          </Paragraph>
          <Paragraph>
            Pro umístění modelů ve virtuálním světě a reakci na stisknutí jejich
            tlačítek je ještě nutné vybrat input systém - unity legacy, nebo
            nový input systém. Zde jsem se rozhodla nepoužít ani jeden a místo
            nich použít třídy z namespacu UnityEngine.XR, neboť mi umožňují
            přímo přečíst stav ovladačů v mnou stanoveném momentě bez přílišných
            ceremonií.
          </Paragraph>
          <Paragraph>
            Čtení vstupu z XR.InputDevice jsem zabalila do jedné třídy
            IsblXRDevice, která obsahuje getter pro každé tlačítko dostupné na
            Vive Wands, nebo na ovladačích Oculus Touch. Pro tlačítka, která se
            jinak jmenují, ale mají stejnou funkci (Menu a MenuButton) obsahuje
            jenom jeden getter a pro chybějící tlačítka generuje rozumnou
            fallback hodnotu, nebo vyhodí výjimku. Z této třídy čte třída
            IsblStaticXRDevice, která přímo v sobě ukládá stav tlačítek - toto
            je pro přenos po síti. Všechny interakce s ovladači probíhají přes
            IsblStaticXRDevice, což znamená, že fungují nezávisle na tom, jestli
            jsou lokální nebo vzdálené.
          </Paragraph>
        </Section>
        <Section title="Online část" no={3}>
          <Paragraph>Pro komunikaci mezi klienty jsem se rozhodla...</Paragraph>
          <TODO>
            TODO, WebSockets (works everywhere, possible to encrypt, well
            defined, but TCP with head-of-line blocking), deno (because
            compile), miniflare, cloudflare workers (because edge). UDP/TCP
            possible future, but seems to work. Velocity/angular velocity delta
            time extrapolation. Same frame updates. Where to transform? Server
            or client-side. Need to transform position, rotation, but also
            velocity and angular velocity.
          </TODO>
        </Section>
        <Section title="Communication protocol" no={4}>
          <Paragraph title="Configuration" first>
            JSON, needs reliable transport. Only sent on change, received upon
            change or new connection. Contains information about layout of data
            messages.
            <TODO>
              TODO: describe json messages once I feel like I won’t be changing
              them.
            </TODO>
            <ParagraphTitle>action: calibration</ParagraphTitle>
            calibrations[]: ... Must never arrive before the “device info”
            message.
          </Paragraph>
          <ParagraphTitle>Uploading data to server</ParagraphTitle>
          <Paragraph>
            Binary message. WebSockets do have flag for that, so there is no
            need to distinguish them and in the UDP future there will be fully
            separate connection there, so no need to distinguish those either.
          </Paragraph>
          <Paragraph>
            Everything is little-endian (C# implementation note: use
            System.Buffers.Binary .BinaryPrimitives instead of
            System.BitConverter to make sure endianness is correct).
          </Paragraph>
          <Paragraph>
            Some integers are written as 7BitEncodedInt which is same as in C#’s
            binary writer[12]. It makes sure that smaller integers are encoded
            as fewer bytes with the tradeoff that large integers ({'>= '}
            2^28) take five bytes.
          </Paragraph>
          <Paragraph>
            Int32 Client ID. It is associated with the connection which means
            that it does not need to be there, but UDP does not have any concept
            of connections so I’ll put it there anyway. Also makes the encoding
            more symmetrical.
          </Paragraph>
          <TODO>Port remaining docs, figure out formatting</TODO>
          <Paragraph title="Receiving data from server">
            Each message can contain data about multiple clients. If data about
            a client is not present it means that data did not change (not a
            disconnect).
          </Paragraph>
          <Paragraph>
            Int32 Number of clients For each client: format same as messages
            sent to server
          </Paragraph>
        </Section>
        <Section title="Device encoding" no={5}>
          <TODO>Port from google docs, figure out formatting</TODO>
        </Section>
        <Section title="Note about compression" no={6}>
          <Paragraph>
            Size of data being uploaded to the server from each client depends
            on the number of devices connected and how much data is available
            for each device. It could be reduced by skipping redundant data
            (left eye is usually only transformed center eye for example).
            Another option to reduce the size of messages would be to compress
            them. For example, currently data upload for HTC Vive with both
            controllers connected uses 476 bytes. If I applied Brotli
            compression on each message separately it would reduce the size to
            less or equal to 327 bytes in my experiment.
          </Paragraph>
          <Paragraph>
            Data upload does not need to be compressed, because even with
            overhead of WebSockets it fits within the usual minimum MTU of 500
            bytes which means that on most networks it should be transmitted as
            a single non-fragmented packet. Data download from the server is
            different because the size is the above multiplied by number of
            clients. So this would be a good improvement to the algorithm.
          </Paragraph>
          <Paragraph>
            I did not implement it mostly because of simplicity of
            implementation since I have 3 different places of which 2 would have
            to implement compression and all 3 would have to implement
            decompression. Also the server runs in two distinct runtime
            environments which limits libraries available for use without
            complicating its build process.
          </Paragraph>
        </Section>
      </Chapter>
      <Chapter title="References">
        <Paragraph>
          {`
          [1]: https://developer.oculus.com/blog/oculus-all-in-on-openxr-deprecates-proprietary-apis/
          [2]: https://unity.com/
          [3]: https://www.unrealengine.com/en-US/unreal-engine-5
          [4]: https://www.roadtovr.com/steam-survey-vr-headsets-on-steam-data-july-2021/
          [5]: https://github.com/alvr-org/alvr
          [6]: https://www.amd.com/en/technologies/radeon-software-relive-v
          [7]: https://github.com/immersive-web/webxr-input-profiles/tree/main/packages/assets
          [8]: https://github.com/immersive-web/webxr-input-profiles/issues/200
          [9]: https://github.com/atteneder/glTFast
          [10]: https://openupm.com/
          [11]: https://blog.vive.com/us/2021/11/11/introducing-new-features-vive-focus-3/
          [12]: https://docs.microsoft.com/en-us/dotnet/api/system.io.binarywriter.write7bitencodedint
          [13]: https://sbtron.github.io/makeglb/
          [14]: https://techcommunity.microsoft.com/t5/mixed-reality-blog/introduction-to-openxr-explorer/ba-p/2733927
          `
            .trim()
            .split('\n')
            .map((v) => v.trim().split(' '))
            .map(([id, link]) => (
              <>
                {id} <pdf.Link src={link}>{link}</pdf.Link>
                {'\n'}
              </>
            ))}
        </Paragraph>
      </Chapter>
    </PDFDocument>
  )
}
