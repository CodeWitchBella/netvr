import { Component } from 'react'
import { Button } from './design'

const showErrorOverlay = (err: Error) => {
  // must be within function call because that's when the element is defined for sure.
  const ErrorOverlay = customElements.get('vite-error-overlay')
  // don't open outside vite environment
  if (!ErrorOverlay) {
    return
  }
  if (err.stack) {
    err.stack = err.stack
      ?.replaceAll?.(
        window.location.protocol + '//' + window.location.host + '/',
        '/',
      )
      .replace(/\?(import&)?t=[0-9]+/, '')
  }
  const overlay = new ErrorOverlay(err)
  document.body
    .querySelectorAll('vite-error-overlay')
    .forEach((o) => o.remove())
  document.body.appendChild(overlay)
}

type State = { hasError: boolean }
export class ErrorBoundary extends Component<{}, State> {
  state: State = { hasError: false }

  static getDerivedStateFromError(error: any): State {
    return { hasError: true }
  }
  componentDidCatch(error: any, errorInfo: any) {
    showErrorOverlay(error)
    console.error(error, errorInfo)
  }
  reset = () => this.setState({ hasError: false })

  render() {
    if (this.state.hasError) {
      // You can render any custom fallback UI
      return (
        <>
          <h1>Something went wrong.</h1>
          <Button type="button" onClick={this.reset}>
            Reset
          </Button>
        </>
      )
    }
    return this.props.children
  }
}
