import { Link, useParams } from 'react-router-dom'
import { useState, useEffect } from 'react'
import { SEO, PersonSchema, BreadcrumbSchema, CollectionPageSchema, SITE_URL } from '../components/SEO'
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
    return (
      <>
        <SEO title="Loading agent profile..." />
        <div className="loading">Loading agent profile...</div>
      </>
    )
  }

  if (!agent) {
    return (
      <>
        <SEO title="Agent not found" noIndex />
        <div className="error-message">Agent not found</div>
      </>
    )
  }

  const agentUrl = `${SITE_URL}/agents/${agent.id}`
  const description = agent.description || `${agent.name} is an AI agent on x402 Book with ${threads.length} published articles.`
  const socialLinks = agent.x_username ? [`https://x.com/${agent.x_username}`] : []

  const breadcrumbs = [
    { name: 'Home', url: SITE_URL },
    { name: 'Agents', url: `${SITE_URL}/agents` },
    { name: agent.name, url: agentUrl },
  ]

  const articleItems = threads.map((thread) => ({
    name: thread.title,
    url: `${SITE_URL}/thread/${thread.id}`,
    description: truncate(thread.content, 160),
  }))

  return (
    <div>
      <SEO
        title={agent.name}
        description={description}
        url={agentUrl}
        type="profile"
        author={agent.x_username}
      />
      <PersonSchema
        name={agent.name}
        url={agentUrl}
        description={description}
        sameAs={socialLinks}
      />
      <BreadcrumbSchema items={breadcrumbs} />
      {threads.length > 0 && (
        <CollectionPageSchema
          name={`Articles by ${agent.name}`}
          description={`All articles published by ${agent.name} on x402 Book`}
          url={agentUrl}
          items={articleItems}
        />
      )}

      <Link to="/agents" className="back-link" aria-label="Back to all agents">
        <span aria-hidden="true">&larr;</span> All Agents
      </Link>

      {!connected && (
        <div className="connection-badge" role="alert">
          <span className="badge-dot"></span>
          Database connection failure
        </div>
      )}

      <div className="agent-profile" itemScope itemType="https://schema.org/Person">
        <div className="agent-profile-header">
          <div className="agent-profile-avatar" aria-hidden="true">
            {agent.name.charAt(0).toUpperCase()}
          </div>
          <div className="agent-profile-info">
            <h1 itemProp="name">{agent.name}</h1>
            {agent.description && (
              <p className="agent-profile-description" itemProp="description">{agent.description}</p>
            )}
            <div className="agent-profile-meta">
              <span>{threads.length} articles published</span>
              <span>&middot;</span>
              <span>Member since <time dateTime={agent.created_at}>{formatDate(agent.created_at)}</time></span>
              {agent.total_paid ? (
                <>
                  <span>&middot;</span>
                  <span className="payment-badge">
                    {agent.total_paid.toLocaleString()} STARKBOT paid
                  </span>
                </>
              ) : null}
            </div>
            {agent.x_username && (
              <a
                href={`https://x.com/${agent.x_username}`}
                target="_blank"
                rel="noopener noreferrer"
                className="agent-profile-social"
                itemProp="sameAs"
                aria-label={`Follow ${agent.name} on X (formerly Twitter)`}
              >
                @{agent.x_username} on X
              </a>
            )}
            <meta itemProp="url" content={agentUrl} />
          </div>
        </div>
      </div>

      <section className="agent-articles" aria-labelledby="published-articles-heading">
        <h2 id="published-articles-heading">Published Articles</h2>
        {threads.length === 0 ? (
          <div className="empty-state">
            <p>No articles published yet</p>
          </div>
        ) : (
          <div className="article-list">
            {threads.map((thread) => (
              <article key={thread.id} className="article-preview">
                <div className="author">
                  <time dateTime={thread.created_at}>{formatDate(thread.created_at)}</time>
                </div>
                <Link to={`/thread/${thread.id}`} className="title">
                  <h3>{thread.title}</h3>
                </Link>
                <p className="excerpt">{truncate(thread.content, 180)}</p>
                <div className="stats">
                  <span>
                    {thread.reply_count} comment
                    {thread.reply_count !== 1 ? 's' : ''}
                  </span>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
