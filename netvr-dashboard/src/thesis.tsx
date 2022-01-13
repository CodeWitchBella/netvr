import Thesis from '@isbl/thesis'
import { useReducer } from 'react'
import { bibliography, chapters, files } from './thesis-text/chapters'

export default function ThesisRenderer() {
  const [config, setConfig] = useReducer(
    (state: Config, partial: Partial<Config>): Config => {
      const nextState = { ...state, ...partial }
      localStorage.setItem('thesis-config', JSON.stringify(nextState))
      return nextState
    },
    null,
    (): Config => ({
      ...defaultConfig,
      ...JSON.parse(localStorage.getItem('thesis-config') || '{}'),
    }),
  )

  return (
    <>
      <ThesisConfig
        config={config}
        setConfig={setConfig}
        chapters={chapters.map((v) => (typeof v === 'string' ? v : v[0]))}
      />
      <Thesis
        bibliography={bibliography}
        chapters={chapters}
        onlyChapter={config.onlyChapter}
        production={config.production}
        useBuiltIn={config.useBuiltIn}
        files={files}
      />
    </>
  )
}
type Config = {
  useBuiltIn: boolean
  production: boolean
  onlyChapter: false | string
}
const defaultConfig: Config = {
  useBuiltIn: false,
  production: true,
  onlyChapter: false,
}

function ThesisConfig({
  config,
  setConfig,
  chapters,
}: {
  config: Config
  setConfig: (cfg: Partial<Config>) => void
  chapters: readonly string[]
}) {
  return (
    <div
      style={{
        gap: 8,
        display: 'flex',
        padding: 4,
        borderBottom: '1px solid gray',
        alignItems: 'center',
      }}
    >
      <div>Config:</div>

      <label>
        <input
          type="checkbox"
          defaultChecked={config.useBuiltIn}
          onChange={(event) => {
            setTimeout(
              () => void setConfig({ useBuiltIn: event.target.checked }),
              0,
            )
          }}
        />{' '}
        use built-in viewer
      </label>
      <label>
        <input
          type="checkbox"
          defaultChecked={config.production}
          onChange={(event) => {
            setTimeout(
              () => void setConfig({ production: event.target.checked }),
              0,
            )
          }}
        />{' '}
        only final-ready
      </label>
      <label>
        <select
          defaultValue={config.onlyChapter || ''}
          onChange={(event) => {
            setTimeout(
              () => void setConfig({ onlyChapter: event.target.value }),
              0,
            )
          }}
        >
          <option value="">Select chapter</option>
          <option value="technical">technical</option>
          {chapters.map((c) => (
            <option value={c} key={c}>
              {c}
            </option>
          ))}
        </select>
      </label>
    </div>
  )
}
