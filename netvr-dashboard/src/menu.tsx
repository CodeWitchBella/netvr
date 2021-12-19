import { NavLink } from 'react-router-dom'

export function Menu() {
  return (
    <nav
      style={{
        display: 'flex',
        gap: 8,
        padding: 8,
        background: 'black',
        color: 'white',
        fontFamily: 'sans-serif',
      }}
    >
      <NavLink
        to="/"
        style={(a) => ({
          textDecoration: a.isActive ? 'underline' : 'none',
          color: 'white',
        })}
      >
        Dashboard
      </NavLink>
      <NavLink
        to="/thesis"
        style={(a) => ({
          textDecoration: a.isActive ? 'underline' : 'none',
          color: 'white',
        })}
      >
        Thesis
      </NavLink>
    </nav>
  )
}
