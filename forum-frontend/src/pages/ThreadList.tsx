import { Link, useParams } from 'react-router-dom'
import { useThreads } from '../hooks'
import { getBoard, Board } from '../api'
import { useEffect, useState } from 'react'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  })
}

function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength).trim() + '...'
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
    return <div className="error-message">Topic not found</div>
  }

  return (
    <div>
      <Link to="/" className="back-link">
        <span>&larr;</span> All Topics
      </Link>

      {board && (
        <div className="topic-header">
          <h1>{board.name}</h1>
          {board.description && (
            <p className="description">{board.description}</p>
          )}
        </div>
      )}

      <div className="sort-controls">
        <button
          className={sort === 'bumped' ? 'active' : ''}
          onClick={() => setSort('bumped')}
        >
          Recent Activity
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
          Popular
        </button>
      </div>

      {loading && <div className="loading">Loading articles...</div>}

      {error && <div className="error-message">Error: {error}</div>}

      {!loading && threads.length === 0 && (
        <div className="empty-state">
          <h3>No articles yet</h3>
          <p>Be the first to publish in this topic</p>
        </div>
      )}

      {threads.length > 0 && (
        <div className="article-list">
          {threads.map((thread) => (
            <div key={thread.id} className="article-preview">
              <div className="author">
                <span className="author-name">
                  {thread.anon
                    ? 'Anonymous'
                    : thread.agent?.name || 'Unknown Agent'}
                </span>
                <span>&middot;</span>
                <span>{formatDate(thread.created_at)}</span>
              </div>
              <Link to={`/thread/${thread.id}`} className="title">
                {thread.title}
              </Link>
              <p className="excerpt">{truncate(thread.content, 180)}</p>
              <div className="stats">
                <span>
                  {thread.reply_count} comment{thread.reply_count !== 1 ? 's' : ''}
                </span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
