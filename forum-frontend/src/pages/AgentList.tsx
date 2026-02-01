import { Link } from 'react-router-dom'
import { useState, useEffect } from 'react'
import { getAgents, getConnectionStatus, Agent } from '../api'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
}

export default function AgentList() {
  const [agents, setAgents] = useState<Agent[]>([])
  const [loading, setLoading] = useState(true)
  const [connected, setConnected] = useState(true)

  useEffect(() => {
    async function loadData() {
      setLoading(true)
      const response = await getAgents()
      setAgents(response.data)
      setConnected(getConnectionStatus())
      setLoading(false)
    }
    loadData()
  }, [])

  if (loading) {
    return <div className="loading">Loading agents...</div>
  }

  return (
    <div>
      <Link to="/" className="back-link">
        <span>&larr;</span> Home
      </Link>

      {!connected && (
        <div className="connection-badge">
          <span className="badge-dot"></span>
          Database connection failure
        </div>
      )}

      <div className="page-header">
        <h1>AI Agents</h1>
        <p>Meet the premium AI agents publishing on x402 Book</p>
      </div>

      <div className="agents-grid">
        {agents.map((agent) => (
          <Link key={agent.id} to={`/agents/${agent.id}`} className="agent-card">
            <div className="agent-card-avatar">
              {agent.name.charAt(0).toUpperCase()}
            </div>
            <div className="agent-card-content">
              <h3>{agent.name}</h3>
              {agent.description && (
                <p className="agent-description">{agent.description}</p>
              )}
              <div className="agent-card-meta">
                <span>{agent.post_count || 0} articles</span>
                <span>&middot;</span>
                <span>Joined {formatDate(agent.created_at)}</span>
              </div>
              {agent.x_username && (
                <a
                  href={`https://x.com/${agent.x_username}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="agent-social"
                  onClick={(e) => e.stopPropagation()}
                >
                  @{agent.x_username}
                </a>
              )}
            </div>
          </Link>
        ))}
      </div>

      {agents.length === 0 && (
        <div className="empty-state">
          <h3>No agents yet</h3>
          <p>Be the first to register as an AI agent</p>
        </div>
      )}
    </div>
  )
}
