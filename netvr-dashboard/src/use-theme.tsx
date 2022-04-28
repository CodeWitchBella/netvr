import * as base16 from 'base16'
import { invertTheme } from 'react-base16-styling'
import type { Base16Theme } from 'react-base16-styling'
import {
  useState,
  useEffect,
  createContext,
  PropsWithChildren,
  useMemo,
  memo,
} from 'react'
import { useLocalStorage } from './utils'
import { useContext } from 'react'
import { Pane, Select } from './design'

const isValidTheme = (v: unknown): v is keyof typeof base16 =>
  typeof v === 'string' && v in base16

const isValidThemeVersion = (v: unknown): v is 'dark' | 'light' | 'system' =>
  v === 'dark' || v === 'light' || v === 'system'

function useSystemTheme() {
  const query = '(prefers-color-scheme: light)'
  const [light, setLight] = useState(window.matchMedia(query).matches)
  useEffect(() => {
    const match = window.matchMedia(query)
    match.addEventListener('change', listener)
    return () => match.removeEventListener('change', listener)
    function listener() {
      setLight(match.matches)
    }
  }, [])
  return light ? 'light' : 'dark'
}

const ctx = createContext<ReturnType<typeof useThemeData> | null>(null)

function useThemeData() {
  const [themeName, setThemeName] = useLocalStorage(
    'theme',
    'monokai',
    isValidTheme,
  )
  const [preferredThemeVersion, setThemeVersion] = useLocalStorage(
    'themeVersion',
    'system',
    isValidThemeVersion,
  )
  const systemThemeVersion = useSystemTheme()
  return useMemo(() => {
    const themeVersion =
      preferredThemeVersion === 'system'
        ? systemThemeVersion
        : preferredThemeVersion
    const baseTheme = base16[themeName]
    return {
      resolved: (themeVersion === 'light'
        ? invertTheme(baseTheme)
        : baseTheme) as any as Base16Theme,
      name: themeName,
      inverted: themeVersion === 'light',
      setName: setThemeName,
      setVersion: setThemeVersion,
      version: preferredThemeVersion,
    }
  }, [
    preferredThemeVersion,
    setThemeName,
    setThemeVersion,
    systemThemeVersion,
    themeName,
  ])
}

export function ThemeProvider({ children }: PropsWithChildren<{}>) {
  return <ctx.Provider value={useThemeData()}>{children}</ctx.Provider>
}

export function useTheme() {
  const theme = useContext(ctx)
  if (!theme) throw new Error('No theme context')
  return theme
}

export const ThemeSelector = memo(function ThemeSelector() {
  const theme = useTheme()
  return (
    <Pane title="Visual settings" id="theme">
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        <label>
          Theme:{' '}
          <Select
            value={theme.name}
            onChange={(event) => {
              theme.setName(event.target.value)
            }}
          >
            {Object.keys(base16).map((k) => (
              <option value={k} key={k}>
                {k[0].toUpperCase() + k.slice(1)}
              </option>
            ))}
          </Select>
        </label>
        <br />
        <label>
          Version:{' '}
          <Select
            value={theme.version}
            onChange={(event) => {
              theme.setVersion(event.target.value)
            }}
          >
            <option value="system">Follow system</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </Select>
        </label>
        <div
          style={{
            display: 'flex',
            marginTop: 4,
            borderRadius: 4,
            overflow: 'hidden',
            borderColor: theme.resolved.base02,
            borderWidth: 1,
            borderStyle: 'solid',
          }}
        >
          {Object.entries(theme.resolved)
            .filter(([k]) => k.startsWith('base'))
            .map(([k, v]) => (
              <div key={k} style={{ background: v, flexGrow: 1 }}>
                <div style={{ paddingBottom: '100%', position: 'relative' }}>
                  <div
                    style={{
                      position: 'absolute',
                      textAlign: 'center',
                      left: 0,
                      right: 0,
                      top: '50%',
                      transform: 'translateY(-50%)',
                      color:
                        Number.parseInt(v.substring(1, 3), 16) +
                          Number.parseInt(v.substring(3, 5), 16) +
                          Number.parseInt(v.substring(5, 7), 16) >
                        128 * 3
                          ? 'black'
                          : 'white',
                    }}
                  >
                    {k.slice(5)}
                  </div>
                </div>
              </div>
            ))}
        </div>
      </div>
    </Pane>
  )
})
