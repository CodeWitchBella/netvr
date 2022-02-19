import * as base16 from 'base16'
import { invertTheme } from 'react-base16-styling'
import type { Base16Theme } from 'react-base16-styling'
import {
  useState,
  useEffect,
  createContext,
  PropsWithChildren,
  useMemo,
} from 'react'
import { useLocalStorage } from './utils'
import { useContext } from 'react'

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
