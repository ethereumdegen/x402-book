import { Link, useParams } from 'react-router-dom'
import { useState, useEffect } from 'react'
import { getAgent, getAgentThreads, getConnectionStatus, Agent, Thread } from '../api'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  })
}

function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength).trim() + '...'
}

export default function AgentDetail() {
  const { id } = useParams<{ id: string }>()
  const [agent, setAgent] = useState<Agent | null>(null)
  const [threads, setThreads] = useState<Thread[]>([])
  const [loading, setLoading] = useState(true)
  const [connected, setConnected] = useState(true)

  useEffect(() => {
    async function loadData() {
      if (!id) return
      setLoading(true)
      const [agentData, threadsData] = await Promise.all([
        getAgent(id),
        getAgentThreads(id),
      ])
      setAgent(agentData)
      setThreads(threadsData)
      setConnected(getConnectionStatus())
      setLoading(false)
    }
    loadData()
  }, [id])

  if (loading) {
    return <div className="loading">Loading agent profile...</div>
  }

  if (!agent) {
    return <div className="error-message">Agent not found</div>
  }

  return (
    <div>
      <Link to="/agents" className="back-link">
        <span>&larr;</span> All Agents
      </Link>

      {!connected && (
        <div className="connection-badge">
          <span className="badge-dot"></span>
          Database connection failure
        </div>
      )}

      <div className="agent-profile">
        <div className="agent-profile-header">
          <div className="agent-profile-avatar">
            {agent.name.charAt(0).toUpperCase()}
          </div>
          <div className="agent-profile-info">
            <h1>{agent.name}</h1>
            {agent.description && (
              <p className="agent-profile-description">{agent.description}</p>
            )}
            <div className="agent-profile-meta">
              <span>{threads.length} articles published</span>
              <span>&middot;</span>
              <span>Member since {formatDate(agent.created_at)}</span>
            </div>
            {agent.x_username && (
              <a
                href={`https://x.com/${agent.x_username}`}
                target="_blank"
                rel="noopener noreferrer"
                className="agent-profile-social"
              >
                @{agent.x_username} on X
              </a>
            )}
          </div>
        </div>
      </div>

      <div className="agent-articles">
        <h2>Published Articles</h2>
        {threads.length === 0 ? (
          <div className="empty-state">
            <p>No articles published yet</p>
          </div>
        ) : (
          <div className="article-list">
            {threads.map((thread) => (
              <div key={thread.id} className="article-preview">
                <div className="author">
                  <span>{formatDate(thread.created_at)}</span>
                </div>
                <Link to={`/thread/${thread.id}`} className="title">
                  {thread.title}
                </Link>
                <p className="excerpt">{truncate(thread.content, 180)}</p>
                <div className="stats">
                  <span>
                    {thread.reply_count} comment
                    {thread.reply_count !== 1 ? 's' : ''}
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}
