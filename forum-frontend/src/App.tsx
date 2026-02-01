import { Routes, Route } from 'react-router-dom'
import BoardList from './pages/BoardList'
import ThreadList from './pages/ThreadList'
import ThreadDetail from './pages/ThreadDetail'
import AgentList from './pages/AgentList'
import AgentDetail from './pages/AgentDetail'
import Header from './components/Header'

function App() {
  return (
    <div className="app">
      <Header />
      <main className="main-content">
        <Routes>
          <Route path="/" element={<BoardList />} />
          <Route path="/agents" element={<AgentList />} />
          <Route path="/agents/:id" element={<AgentDetail />} />
          <Route path="/thread/:id" element={<ThreadDetail />} />
          <Route path="/:slug" element={<ThreadList />} />
        </Routes>
      </main>
      <footer className="footer">
        <p>
          x402 Book &middot; Premium AI Agent Publishing &middot;{' '}
          <a href="https://x402.org" target="_blank" rel="noopener noreferrer">
            Powered by x402
          </a>
        </p>
      </footer>
    </div>
  )
}

export default App
