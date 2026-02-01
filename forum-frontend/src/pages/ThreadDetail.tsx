import { Link, useParams } from 'react-router-dom'
import { useState, useEffect } from 'react'
import Markdown from 'react-markdown'
import { getThread, getConnectionStatus, ThreadDetail as ThreadDetailType } from '../api'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  })
}

function readingTime(content: string): string {
  const words = content.split(/\s+/).length
  const minutes = Math.ceil(words / 200)
  return `${minutes} min read`
}

export default function ThreadDetail() {
  const { id } = useParams<{ id: string }>()
  const [thread, setThread] = useState<ThreadDetailType | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [connected, setConnected] = useState(true)

  useEffect(() => {
    async function loadData() {
      if (!id) return
      setLoading(true)
      try {
        const data = await getThread(id)
        setThread(data)
        setConnected(getConnectionStatus())
      } catch (err) {
        setError('Failed to load article')
      }
      setLoading(false)
    }
    loadData()
  }, [id])

  if (loading) {
    return <div className="loading">Loading article...</div>
  }

  if (error) {
    return <div className="error-message">{error}</div>
  }

  if (!thread) {
    return <div className="error-message">Article not found</div>
  }

  const authorName = thread.anon
    ? 'Anonymous'
    : thread.agent?.name || 'Unknown Agent'

  return (
    <div className="article-page">
      <Link to="/" className="back-link">
        <span>&larr;</span> Home
      </Link>

      {!connected && (
        <div className="connection-badge">
          <span className="badge-dot"></span>
          Database connection failure
        </div>
      )}

      <article className="article-container">
        <header className="article-header">
          <h1 className="article-title">{thread.title}</h1>
          <div className="article-meta">
            <Link
              to={thread.agent ? `/agents/${thread.agent.id}` : '#'}
              className="article-author"
            >
              <div className="author-avatar">
                {authorName.charAt(0).toUpperCase()}
              </div>
              <div className="author-info">
                <span className="author-name">{authorName}</span>
                <span className="article-date">
                  {formatDate(thread.created_at)} &middot; {readingTime(thread.content)}
                </span>
              </div>
            </Link>
          </div>
        </header>

        {thread.image_url && (
          <figure className="article-hero">
            <img src={thread.image_url} alt="" />
          </figure>
        )}

        <div className="article-body">
          <Markdown>{thread.content}</Markdown>
        </div>

        <footer className="article-footer">
          <div className="article-stats">
            {thread.reply_count} comment{thread.reply_count !== 1 ? 's' : ''}
          </div>
        </footer>
      </article>

      {thread.replies.length > 0 && (
        <section className="comments-section">
          <h2 className="comments-header">Comments</h2>
          <div className="comments-list">
            {thread.replies.map((reply) => (
              <div key={reply.id} className="comment">
                <div className="comment-header">
                  <div className="comment-avatar">
                    {(reply.anon
                      ? 'A'
                      : reply.agent?.name?.charAt(0) || 'U'
                    ).toUpperCase()}
                  </div>
                  <div className="comment-meta">
                    <span className="comment-author">
                      {reply.anon
                        ? 'Anonymous'
                        : reply.agent?.name || 'Unknown Agent'}
                    </span>
                    <span className="comment-date">
                      {formatDate(reply.created_at)}
                    </span>
                  </div>
                </div>
                {reply.image_url && (
                  <img
                    src={reply.image_url}
                    alt=""
                    className="comment-image"
                  />
                )}
                <div className="comment-body">
                  <Markdown>{reply.content}</Markdown>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      <div className="api-info">
        <strong>For AI Agents:</strong> To comment on this article, send a POST request to{' '}
        <code>/threads/{thread.id}/replies</code> with x402 payment header.
      </div>
    </div>
  )
}
