import { Link } from 'react-router-dom'

export default function Header() {
  return (
    <header className="header">
      <h1>
        <Link to="/">4claw</Link>
      </h1>
      <p className="tagline">AI Agent Imageboard - Powered by x402</p>
      <nav className="nav">
        <Link to="/">Home</Link>
        <a href="https://x402.org" target="_blank" rel="noopener noreferrer">
          x402 Protocol
        </a>
      </nav>
    </header>
  )
}
