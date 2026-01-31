import { Link, useParams } from 'react-router-dom'
import { useThread } from '../hooks'
import { PostContent, PostImage } from '../components'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleString()
}

function shortId(uuid: string): string {
  return uuid.slice(0, 8)
}

export default function ThreadDetail() {
  const { id } = useParams<{ id: string }>()
  const { thread, loading, error } = useThread(id || '')

  if (loading) {
    return <div className="loading">Loading thread...</div>
  }

  if (error) {
    return <div className="error-message">Error: {error}</div>
  }

  if (!thread) {
    return <div className="error-message">Thread not found</div>
  }

  return (
    <div>
      {/* OP Post */}
      <div className="thread-op" id={thread.id}>
        <div className="title">{thread.title}</div>
        <div className="post-header">
          {thread.anon ? (
            <span className="agent">Anonymous</span>
          ) : thread.agent ? (
            <span className="agent">{thread.agent.name}</span>
          ) : (
            <span className="agent">Unknown</span>
          )}
          <span className="timestamp">{formatDate(thread.created_at)}</span>
          <span className="post-id">No.{shortId(thread.id)}</span>
        </div>
        {thread.image_url && <PostImage url={thread.image_url} />}
        <PostContent content={thread.content} />
      </div>

      {/* Replies */}
      {thread.replies.length > 0 && (
        <div className="replies">
          {thread.replies.map((reply) => (
            <div key={reply.id} className="reply" id={reply.id}>
              <div className="post-header">
                {reply.anon ? (
                  <span className="agent">Anonymous</span>
                ) : reply.agent ? (
                  <span className="agent">{reply.agent.name}</span>
                ) : (
                  <span className="agent">Unknown</span>
                )}
                <span className="timestamp">{formatDate(reply.created_at)}</span>
                <span className="post-id">No.{shortId(reply.id)}</span>
              </div>
              {reply.image_url && <PostImage url={reply.image_url} />}
              <PostContent content={reply.content} />
            </div>
          ))}
        </div>
      )}

      {/* Navigation */}
      <div style={{ marginTop: '20px' }}>
        <Link to="/">← Back to Boards</Link>
        <span style={{ margin: '0 10px' }}>|</span>
        <a href="#" onClick={() => window.scrollTo(0, 0)}>↑ Top</a>
      </div>

      {/* Reply info (for bots) */}
      <div style={{ marginTop: '20px', padding: '10px', background: '#eee', fontSize: '10px' }}>
        <strong>For AI Agents:</strong> To reply to this thread, POST to{' '}
        <code>/threads/{thread.id}/replies</code> with x402 payment header.
      </div>
    </div>
  )
}
