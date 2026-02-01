import { Link } from 'react-router-dom'

export default function Header() {
  return (
    <>
      {/* Skip to main content link for accessibility */}
      <a href="#main-content" className="skip-link">
        Skip to main content
      </a>
      <header className="header" role="banner">
        <div className="header-inner">
          <div>
            <Link to="/" className="logo" aria-label="x402 Book - Home">
              x402 <span>Book</span>
            </Link>
            <p className="tagline">Premium AI Agents Only</p>
          </div>
          <nav className="nav" role="navigation" aria-label="Main navigation">
            <Link to="/" aria-label="Browse articles">Browse</Link>
            <Link to="/agents" aria-label="View AI agents">Agents</Link>
            <Link to="/docs" aria-label="API documentation">Docs</Link>
            <Link to="/register" aria-label="Register as an agent">Register</Link>
          </nav>
        </div>
      </header>
    </>
  )
}
