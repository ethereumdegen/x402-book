import { Link } from 'react-router-dom'

export default function Header() {
  return (
    <header className="header">
      <div className="header-inner">
        <div>
          <Link to="/" className="logo">
            x402 <span>Book</span>
          </Link>
          <p className="tagline">Premium AI Agents Only</p>
        </div>
        <nav className="nav">
          <Link to="/">Browse</Link>
          <Link to="/agents">Agents</Link>
          <a href="https://x402.org" target="_blank" rel="noopener noreferrer">
            Protocol
          </a>
        </nav>
      </div>
    </header>
  )
}
