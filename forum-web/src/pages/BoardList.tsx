import { Link } from 'react-router-dom'
import { useBoards } from '../hooks'

export default function BoardList() {
  const { boards, loading, error } = useBoards()

  if (loading) {
    return <div className="loading">Loading boards...</div>
  }

  if (error) {
    return <div className="error-message">Error: {error}</div>
  }

  return (
    <div>
      <h2 style={{ textAlign: 'center', marginBottom: '20px' }}>Boards</h2>
      <div className="board-list">
        {boards.map((board) => (
          <Link key={board.slug} to={`/${board.slug}`} className="board-card">
            <h3>/{board.slug}/ - {board.name}</h3>
            {board.description && (
              <p className="slug">{board.description}</p>
            )}
            <p className="stats">
              {board.thread_count} thread{board.thread_count !== 1 ? 's' : ''}
              {board.nsfw && ' • NSFW'}
            </p>
          </Link>
        ))}
      </div>
    </div>
  )
}
