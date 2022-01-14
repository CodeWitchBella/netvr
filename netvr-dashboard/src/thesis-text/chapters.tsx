import test from './test.md?raw'
import technicalDesign from './technical-design.md?raw'
import chapter1 from './1-introduction.md?raw'
import chapter2 from './2-analysis.md?raw'
import chapter3 from './3-architecture.md?raw'
import chapter4 from './4-demo.md?raw'
import chapter5 from './5-accuracy.md?raw'
import chapter6 from './6-conclusion.md?raw'
import bib from './bibliography.json?raw'
import chartSvg from './chart.svg?raw'

export const chapters: readonly (
  | readonly [
      id: string,
      data: string,
      extra?: { removeInProduction: boolean; appendix: boolean },
    ]
  | 'toc'
  | 'bibliography'
  | 'begin'
  | {
      id: string
      type: 'split'
      text: readonly [string, string]
      titles: readonly [string, string]
      removeInProduction?: boolean
    }
)[] = [
  declaration(),
  { ...abstract(), removeInProduction: true },
  'toc',
  'begin',
  ['introduction', chapter1],
  ['analysis', chapter2],
  ['architecture', chapter3],
  ['demo', chapter4],
  ['accuracy', chapter5],
  ['conclusion', chapter6],
  'bibliography',
  [
    'technical-design',
    technicalDesign,
    { removeInProduction: true, appendix: true },
  ],
  ['test', test, { removeInProduction: true, appendix: true }],
]

export const files = {
  'quest2-optitrack.png': new URL('quest2-optitrack.png', import.meta.url).href,
  'chart.svg': chartSvg,
  'unity-device-drawer.png': new URL('unity-device-drawer.png', import.meta.url)
    .href,
  'unity-net-drawer.png': new URL('unity-net-drawer.png', import.meta.url).href,
  'web-dashboard.png': new URL('web-dashboard.png', import.meta.url).href,
}

export const bibliography = JSON.parse(bib).bibliography

function declaration() {
  const markdown = String.raw
  return {
    id: 'declaration',
    type: 'split',
    titles: ['Acknowledgements', 'Declaration'],
    text: [
      markdown`
I would like to express my gratitude
to my supervisor, Ing. David Sedláček,
Ph.D., for his guidance over the course of
writing semester project and later of this
thesis. I would also like to thank him and
the Department of Computer Graphics
and Interaction for giving me access to
equipment I needed for the completion
of this thesis.
      `.trim(),
      markdown`
I declare that this thesis represents
my work and that I have listed all the
literature used in the bibliography.

Prague, 20 June 2022

::space

Prohlašuji, že jsem předloženou práci
vypracovala samostatně, a že jsem uved-
la veškerou použitou literaturu.

V Praze, 20. června 2022

:todo[Update dates, both of them]
      `.trim(),
    ],
  } as const
}

function abstract() {
  const markdown = String.raw
  return {
    id: 'abstract',
    type: 'split',
    titles: ['Abstract', 'Abstrakt'],
    text: [
      markdown`
(placeholder, real abstract will be
provided for the thesis) Lorem ipsum
dolor sit amet, consectetur adipiscing
elit. Etiam commodo orci imperdiet vo-
lutpat malesuada. Vestibulum quis mas-
sa tristique, lobortis arcu quis, phare-
tra ligula. Aliquam vestibulum metus
eget sapien porta laoreet. Sed ut po-
suere urna. Sed quis mi hendrerit, cursus
ligula in, luctus tellus. Integer rhon-
cus, mauris in eleifend volutpat, arcu
elit semper ante, a luctus nisi metus
id odio. Suspendisse potenti. Aliquam
nec ante eget arcu sollicitudin vehicula.
Aliquam faucibus, lorem pulvinar tris-
tique blandit, ipsum nunc eleifend est, at
feugiat velit quam vel ante. Sed sapien
libero, volutpat sed mauris quis, mo-
lestie iaculis risus. Morbi pharetra, mi in
fermentum vehicula, nisi ipsum ullam-
corper turpis, finibus feugiat lectus eros
nec turpis. Cras in orci ligula. Aenean
sagittis, velit in ultricies lobortis, augue
justo tempus orci, sit amet iaculis tellus
elit vitae nunc. Etiam elementum sollic-
itudin lorem, eget ornare erat bibendum
non.

::space

**Keywords:** VR, Stuff, Things
      `.trim(),
      markdown`
(placeholder from blabot.cz) Vyšla ať té příslušník světa
nové prozkoumány struktury o ságy rok místnost naši a bude
superstrun jídelny i výstavě od one náš vlhkost pódia
velkým. Tím slunce drží níž i básník zradit
sedmikilometrového sledování vláknité a multi-dimenzionálním
systematicky. Jednom, 80 ℃ kratší ptal ně indickým životním
přetlakovaný i větší vystoupám tím instituce o hladinou šest
psychologických starosta. Amoku kroje v nejprve i sociální
existuje minerálů s potvrzují rozvoji, vznikly pás tisíc
představ klidné kdysi by správní nadšenců hlavě. Nejméně jí
tu chobotnice skákat, oxidu cíl posílily vláken testům. Z
oprášil plyne vědecké třetí jednom ani itálie ekologickou
ohrožení objeveny s vodorovně chorvati, o teoretickým snila
nejlepší z predátorů kterou z zlata božská kanadské. Horečky
k národností! Či EU sága vrátit řadu mohlo z svědčí spouští
testy o zápory? Odlišné nebo marná mám, běžnou kontinentu.
Ně ze včera vlna cestou po polopotopenou. Ať okouzlí, hlavní
klecích zkoušet dosahu s s historkám ochlazení, mým pomocí
od petr rozloučím slunečního skončení kostely, každý pravdou
tj. 1963–1977 starala o reprezentační.

::space

**Klíčová slova:** VR, Stuff, Things

::space

**Překlad názvu:** Sledování více uživatelů VR světa ve sdíleném fyzickém prostoru
      `.trim(),
    ],
  } as const
}
