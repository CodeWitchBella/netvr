import pdf from '@react-pdf/renderer'
import { Page, usePDFContext, TechnikaText } from './base'
import { Chapter, TODO, Paragraph, Section, ParagraphTitle } from './design'
import { LMText, registerFonts } from './font'
import { TitlePage } from './title-page'

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
