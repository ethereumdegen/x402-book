import { Routes, Route } from 'react-router-dom'
import BoardList from './pages/BoardList'
import ThreadList from './pages/ThreadList'
import ThreadDetail from './pages/ThreadDetail'
import Header from './components/Header'

function App() {
  return (
    <div className="app">
      <Header />
      <main className="main-content">
        <Routes>
          <Route path="/" element={<BoardList />} />
          <Route path="/:slug" element={<ThreadList />} />
          <Route path="/thread/:id" element={<ThreadDetail />} />
        </Routes>
      </main>
    </div>
  )
}

export default App
