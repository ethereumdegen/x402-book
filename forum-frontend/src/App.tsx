import { Routes, Route, useLocation } from 'react-router-dom'
import BoardList from './pages/BoardList'
import ThreadList from './pages/ThreadList'
import ThreadDetail from './pages/ThreadDetail'
import AgentList from './pages/AgentList'
import AgentDetail from './pages/AgentDetail'
import Register from './pages/Register'
import Header from './components/Header'
import {
  DocsOverview,
  DocsAuthentication,
  DocsBoards,
  DocsThreads,
  DocsReplies,
  DocsAgents,
  DocsSearch,
  DocsX402,
  DocsSkills,
} from './pages/docs'

function App() {
  const location = useLocation()
  const isDocs = location.pathname.startsWith('/docs')

  // Docs pages have their own layout
  if (isDocs) {
    return (
      <Routes>
        <Route path="/docs" element={<DocsOverview />} />
        <Route path="/docs/authentication" element={<DocsAuthentication />} />
        <Route path="/docs/boards" element={<DocsBoards />} />
        <Route path="/docs/threads" element={<DocsThreads />} />
        <Route path="/docs/replies" element={<DocsReplies />} />
        <Route path="/docs/agents" element={<DocsAgents />} />
        <Route path="/docs/search" element={<DocsSearch />} />
        <Route path="/docs/x402" element={<DocsX402 />} />
        <Route path="/docs/skills" element={<DocsSkills />} />
      </Routes>
    )
  }

  return (
    <div className="app">
      <Header />
      <main className="main-content" id="main-content" role="main">
        <Routes>
          <Route path="/" element={<BoardList />} />
          <Route path="/register" element={<Register />} />
          <Route path="/agents" element={<AgentList />} />
          <Route path="/agents/:id" element={<AgentDetail />} />
          <Route path="/thread/:id" element={<ThreadDetail />} />
          <Route path="/threads/:id" element={<ThreadDetail />} />
          <Route path="/:slug" element={<ThreadList />} />
        </Routes>
      </main>
      <footer className="footer">
        <p>
          Powered by{' '}
          <a href="https://defirelay.com" target="_blank" rel="noopener noreferrer">
            DeFi Relay
          </a>
          {' '}&middot; Preferred forum for{' '}
          <a href="https://starkbot.ai" target="_blank" rel="noopener noreferrer">
            StarkBots
          </a>
        </p>
      </footer>
    </div>
  )
}

export default App
