/** @jsxImportSource @emotion/react */
import * as base16 from 'base16'
import { invertTheme } from 'react-base16-styling'
import type { Base16Theme } from 'react-base16-styling'
import React, {
  useState,
  useEffect,
  createContext,
  PropsWithChildren,
  useMemo,
  memo,
} from 'react'
import { useLocalStorage } from '../utils'
import { useContext } from 'react'
import { Pane, Select, selectionStyle } from './design'
import { css, Global } from '@emotion/react'
import { FullscreenButton } from '../dashboard/fullscreen-button'

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

export function ThemeRoot({ children }: PropsWithChildren<{}>) {
  const data = useThemeData()
  return (
    <>
      <ctx.Provider value={data}>{children}</ctx.Provider>
      <Global
        styles={[
          {
            ':root': {
              ...Object.fromEntries(
                Object.entries(data.resolved).map(([k, v]) =>
                  k.startsWith('base')
                    ? [`--base-${k.slice(5).toLowerCase()}`, v]
                    : [],
                ),
              ),
              background: data.resolved.base01,
              color: data.resolved.base07,
            },
          },
          selectionStyle,
        ]}
      />
    </>
  )
}

const opaque = Symbol()

export function useReprovideTheme() {
  const theme = useThemeInternal()
  return { [opaque]: theme }
}

export function ReprovideTheme(props: {
  children: React.ReactNode
  value: ReturnType<typeof useReprovideTheme>
}) {
  return (
    <ctx.Provider value={props.value[opaque]}>{props.children}</ctx.Provider>
  )
}

function useThemeInternal() {
  const theme = useContext(ctx)
  if (!theme) throw new Error('No theme context')
  return theme
}

export function useTheme() {
  return useThemeInternal().resolved
}

export const ThemeSelector = memo(function ThemeSelector() {
  const theme = useThemeInternal()
  return (
    <Pane title="Visual settings" id="theme" buttons={<FullscreenButton />}>
      <div css={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
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
          css={{
            display: 'flex',
            marginTop: 4,
            borderRadius: 4,
            overflow: 'hidden',
            borderColor: 'var(--base-2)',
            borderWidth: 1,
            borderStyle: 'solid',
          }}
        >
          {Object.entries(theme.resolved)
            .filter(([k]) => k.startsWith('base'))
            .map(([k, v]) => (
              <ThemeColorButton key={k} k={k} v={v} />
            ))}
        </div>
      </div>
    </Pane>
  )
})

function ThemeColorButton({ k, v }: { k: string; v: string }) {
  const [check, setCheck] = useState(0)
  useEffect(() => {
    if (check) {
      const tim = setTimeout(() => {
        setCheck(0)
      }, 1000)
      return () => void clearTimeout(tim)
    }
  }, [check])
  const variable = `var(--base-${k.slice(5).toLowerCase()})`
  return (
    <button
      type="button"
      onClick={() => {
        // click twice to copy plain css, otherwise copy js string
        if (check) navigator.clipboard.writeText(variable)
        else navigator.clipboard.writeText(`'${variable}'`)
        setCheck((v) => v + 1)
      }}
      css={{
        all: 'unset',
        userSelect: 'none',
        flexGrow: 1,
        '&:hover': { filter: 'brightness(0.8) contrast(1.2)' },
      }}
      style={{ background: variable }}
    >
      <div
        css={{
          paddingBottom: '100%',
          position: 'relative',
        }}
      >
        <div
          css={[
            {
              position: 'absolute',
              textAlign: 'center',
              left: 0,
              right: 0,
              top: '50%',
              transform: 'translateY(-50%)',
            },
            Number.parseInt(v.substring(1, 3), 16) +
              Number.parseInt(v.substring(3, 5), 16) +
              Number.parseInt(v.substring(5, 7), 16) >
            128 * 3
              ? css({ color: 'black' })
              : css({ color: 'white' }),
          ]}
        >
          {check ? (check > 1 ? '✔' : "'✔'") : k.slice(5)}
        </div>
      </div>
    </button>
  )
}
