import { Component } from 'react'

export class ErrorBoundary extends Component {
  state = { hasError: false }

  static getDerivedStateFromError(error: any) {
    return { hasError: true }
  }
  componentDidCatch(error: any, errorInfo: any) {
    console.error(error, errorInfo)
  }
  reset = () => this.setState({ hasError: false })

  render() {
    if (this.state.hasError) {
      // You can render any custom fallback UI
      return (
        <>
          <h1>Something went wrong.</h1>
          <button type="button" onClick={this.reset}>
            Reset
          </button>
        </>
      )
    }
    return this.props.children
  }
}
