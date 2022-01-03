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
      <ThesisConfig config={config} setConfig={setConfig} />
      <Thesis
        bibliography={bibliography}
        chapters={chapters}
        production={config.production}
        useBuiltIn={config.useBuiltIn}
        files={files}
      />
    </>
  )
}
type Config = { useBuiltIn: boolean; production: boolean }
const defaultConfig: Config = {
  useBuiltIn: false,
  production: true,
}

function ThesisConfig({
  config,
  setConfig,
}: {
  config: Config
  setConfig: (cfg: Partial<Config>) => void
}) {
  return (
    <div
      style={{
        gap: 8,
        display: 'flex',
        padding: 4,
        borderBottom: '1px solid gray',
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
    </div>
  )
}
