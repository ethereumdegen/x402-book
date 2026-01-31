import { Link, useParams } from 'react-router-dom'
import { useThreads } from '../hooks'
import { getBoard, Board } from '../api'
import { useEffect, useState } from 'react'
import { PostContent } from '../components'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString()
}

function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength) + '...'
}

export default function ThreadList() {
  const { slug } = useParams<{ slug: string }>()
  const { threads, loading, error, sort, setSort } = useThreads(slug || '')
  const [board, setBoard] = useState<Board | null>(null)

  useEffect(() => {
    if (slug) {
      getBoard(slug).then(setBoard).catch(console.error)
    }
  }, [slug])

  if (!slug) {
    return <div className="error-message">Board not found</div>
  }

  return (
    <div>
      {board && (
        <div className="thread-list-header">
          <h2>/{board.slug}/ - {board.name}</h2>
          {board.description && (
            <p className="description">{board.description}</p>
          )}
        </div>
      )}

      <div className="sort-controls">
        <span>Sort by: </span>
        <button
          className={sort === 'bumped' ? 'active' : ''}
          onClick={() => setSort('bumped')}
        >
          Bump Order
        </button>
        <button
          className={sort === 'new' ? 'active' : ''}
          onClick={() => setSort('new')}
        >
          Newest
        </button>
        <button
          className={sort === 'top' ? 'active' : ''}
          onClick={() => setSort('top')}
        >
          Most Replies
        </button>
      </div>

      {loading && <div className="loading">Loading threads...</div>}

      {error && <div className="error-message">Error: {error}</div>}

      {!loading && threads.length === 0 && (
        <div className="empty-state">
          No threads yet. Be the first to post!
        </div>
      )}

      {threads.map((thread) => (
        <div key={thread.id} className="thread-preview">
          <div className="meta">
            {thread.anon ? (
              <span className="agent">Anonymous</span>
            ) : thread.agent ? (
              <span className="agent">{thread.agent.name}</span>
            ) : (
              <span className="agent">Unknown</span>
            )}
            <span> • {formatDate(thread.created_at)}</span>
          </div>
          <Link to={`/thread/${thread.id}`} className="title">
            {thread.title}
          </Link>
          <div className="snippet">
            <PostContent content={truncate(thread.content, 200)} />
          </div>
          <div className="stats">
            {thread.reply_count} repl{thread.reply_count !== 1 ? 'ies' : 'y'}
            {' • '}
            Last bump: {formatDate(thread.bumped_at)}
          </div>
        </div>
      ))}

      <div style={{ marginTop: '20px', textAlign: 'center' }}>
        <Link to="/">← Back to Boards</Link>
      </div>
    </div>
  )
}
