import test from './test.md?raw'
import technicalDesign from './technical-design.md?raw'

export const chapters: readonly (readonly [string, string])[] = Object.entries({
  test,
  technicalDesign,
})

export type Citation = {
  url: string
  authors?: readonly (
    | { firstname: string; surname: string }
    | { group: string }
  )[]
  title?: string
  subtitle?: string
  date?: number | string
  edition?: string
  location?: string
  publisher?: string
  in?: string
  accessed?: string
}

export const citations: { [key: string]: Citation } = {
  'steam-hardware-survey': {
    authors: [{ firstname: 'Ben', surname: 'Lang' }],
    title:
      'VR on Steam Bounces Back to Nearly 2.8 Million Monthly-connected Headsets',
    url: 'https://www.roadtovr.com/steam-survey-vr-headsets-on-steam-data-july-2021/',
    date: 2021,
    publisher: 'Road to VR',
    accessed: '2021-12-29',
  },
  unity: {
    url: 'https://unity.com/',
    title: 'Unity Real-Time Development Platform',
    accessed: '2021-11-16',
    authors: [{ group: 'Unity Technologies' }],
  },
  unreal5: {
    url: 'https://www.unrealengine.com/en-US/unreal-engine-5',
    title: 'Unreal Engine 5 Early Access',
    authors: [{ group: 'Epic Games' }],
    accessed: '2021-11-16',
  },
  alvr: {
    url: 'https://github.com/alvr-org/alvr',
    title: 'ALVR - Air Light VR',
    accessed: '2021-12-29',
  },
  'relive-vr': {
    url: 'https://www.amd.com/en/technologies/radeon-software-relive-vr',
    title: 'AMD Radeon™ ReLive for VR',
    authors: [{ group: 'AMD' }],
    accessed: '2021-12-29',
  },
  gltfast: {
    url: 'https://github.com/atteneder/glTFast',
    title: 'glTFast',
    authors: [{ firstname: 'Andreas', surname: 'Atteneder' }],
    accessed: '2021-12-29',
  },
  openupm: {
    title: 'Open Source Unity Package Registry',
    url: 'https://openupm.com/',
    accessed: '2021-12-29',
  },
  'focus-lbe': {
    url: 'https://blog.vive.com/us/2021/11/11/introducing-new-features-vive-focus-3/',
    title: ' Introducing New Features for VIVE Focus 3',
    authors: [{ firstname: 'Shen', surname: 'Ye' }],
    in: 'VIVE Blog',
    accessed: '2021-11-13',
    date: '2021-11-11',
  },
  '7bit-int': {
    url: 'https://docs.microsoft.com/en-us/dotnet/api/system.io.binarywriter.write7bitencodedint',
    title: 'BinaryWriter.Write7BitEncodedInt(Int32) Method',
    authors: [{ group: 'Microsoft' }],
    accessed: '2021-12-29',
  },
  makeglb: {
    url: 'https://sbtron.github.io/makeglb/',
  },
  'openxr-explorer': {
    url: 'https://techcommunity.microsoft.com/t5/mixed-reality-blog/introduction-to-openxr-explorer/ba-p/2733927',
    authors: [{ group: 'Microsoft' }],
    title: 'Introduction to OpenXR Explorer',
    in: 'Mixed Reality Blog',
    accessed: '2021-12-29',
    date: '2021-09-10',
  },
  'unaligned-controller-issue': {
    url: 'https://github.com/immersive-web/webxr-input-profiles/issues/200',
    title:
      'HTC Vive Wand controller models are offset in relation to SteamVR overlay',
    authors: [{ firstname: 'Isabella', surname: 'Skořepová' }],
    accessed: '2021-12-29',
    date: 2021,
  },
  'controller-models': {
    url: 'https://github.com/immersive-web/webxr-input-profiles/tree/main/packages/assets',
    title: 'WebXR Input Profiles - Assets',
    accessed: '2021-12-29',
  },
  'oculus-openxr': {
    url: 'https://developer.oculus.com/blog/oculus-all-in-on-openxr-deprecates-proprietary-apis/',
    title: 'Oculus All In on OpenXR: Deprecates Proprietary APIs',
    authors: [{ group: 'Oculus VR' }],
    date: '2021-07-23',
  },
}
