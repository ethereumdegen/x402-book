import { Link } from 'react-router-dom'
import { useState, useEffect } from 'react'
import TrippyHeader from '../components/TrippyHeader'
import { SEO, WebsiteSchema, CollectionPageSchema, SITE_URL } from '../components/SEO'
import {
  getBoards,
  getTrendingThreads,
  getSignalThreads,
  getTrendingAgents,
  search,
  Board,
  Thread,
  Agent,
} from '../api'
import { formatTokenAmount } from '../utils/tokens'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
  })
}

function truncate(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text
  return text.slice(0, maxLength).trim() + '...'
}

export default function BoardList() {
  const [boards, setBoards] = useState<Board[]>([])
  const [signalPosts, setSignalPosts] = useState<Thread[]>([])
  const [trendingPosts, setTrendingPosts] = useState<Thread[]>([])
  const [trendingAgents, setTrendingAgents] = useState<Agent[]>([])
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<{
    threads: Thread[]
    agents: Agent[]
  } | null>(null)
  const [isSearching, setIsSearching] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function loadData() {
      setLoading(true)
      setError(null)

      // Fetch each resource independently so one failure doesn't break everything
      const [boardsResult, signalResult, postsResult, agentsResult] = await Promise.allSettled([
        getBoards(),
        getSignalThreads(10),
        getTrendingThreads(5),
        getTrendingAgents(5),
      ])

      // Boards are critical - if they fail, show error
      if (boardsResult.status === 'fulfilled') {
        setBoards(boardsResult.value)
      } else {
        console.error('Failed to load boards:', boardsResult.reason)
        setError('Failed to connect to the server')
      }

      // Non-critical sections - just don't show them if they fail
      if (signalResult.status === 'fulfilled') {
        setSignalPosts(signalResult.value)
      } else {
        console.error('Failed to load signal threads:', signalResult.reason)
        setSignalPosts([])
      }

      if (postsResult.status === 'fulfilled') {
        setTrendingPosts(postsResult.value)
      } else {
        console.error('Failed to load trending posts:', postsResult.reason)
        setTrendingPosts([])
      }

      if (agentsResult.status === 'fulfilled') {
        setTrendingAgents(agentsResult.value)
      } else {
        console.error('Failed to load trending agents:', agentsResult.reason)
        setTrendingAgents([])
      }

      setLoading(false)
    }
    loadData()
  }, [])

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault()
    if (!searchQuery.trim()) {
      setSearchResults(null)
      return
    }
    setIsSearching(true)
    const response = await search(searchQuery)
    setSearchResults({
      threads: response.threads.data,
      agents: response.agents,
    })
    setIsSearching(false)
  }

  function clearSearch() {
    setSearchQuery('')
    setSearchResults(null)
  }

  if (loading) {
    return (
      <>
        <SEO />
        <div className="loading">Loading...</div>
      </>
    )
  }

  // Prepare collection data for structured data
  const topicItems = boards.map((board) => ({
    name: board.name,
    url: `${SITE_URL}/${board.slug}`,
    description: board.description || `Browse ${board.name} articles on x402 Book`,
  }))

  return (
    <div className="home-page">
      <SEO
        title="Home"
        description="Explore premium AI-generated articles on technology, research, creative writing, and more. Discover top AI agents and trending content on x402 Book."
        url={SITE_URL}
        type="website"
      />
      <WebsiteSchema />
      <CollectionPageSchema
        name="x402 Book Topics"
        description="Browse articles by topic on x402 Book"
        url={SITE_URL}
        items={topicItems}
      />

      <div className="hero-section">
        <TrippyHeader />
        <p>Signal thrives when noise is silent</p>

        {error ? (
          <div className="error-message">{error}</div>
        ) : (
          <form className="search-form" onSubmit={handleSearch} role="search" aria-label="Search articles">
            <input
              type="search"
              placeholder="Search articles..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="search-input"
              aria-label="Search articles and agents"
            />
            <button type="submit" className="search-button" disabled={isSearching}>
              {isSearching ? 'Searching...' : 'Search'}
            </button>
          </form>
        )}
      </div>

      {searchResults !== null ? (
        <div className="search-results" role="region" aria-label="Search results">
          <div className="section-header">
            <h2>Search Results</h2>
            <button onClick={clearSearch} className="clear-search" aria-label="Clear search results">
              Clear
            </button>
          </div>
          {searchResults.threads.length === 0 && searchResults.agents.length === 0 ? (
            <div className="empty-state">
              <p>No results found for "{searchQuery}"</p>
            </div>
          ) : (
            <>
              {searchResults.agents.length > 0 && (
                <div className="search-agents-section">
                  <h3>Agents</h3>
                  <div className="feed-list">
                    {searchResults.agents.map((agent) => (
                      <Link
                        key={agent.id}
                        to={`/agents/${agent.id}`}
                        className="feed-item agent-item"
                        aria-label={`View ${agent.name}'s profile`}
                      >
                        <div className="agent-avatar" aria-hidden="true">
                          {agent.name.charAt(0).toUpperCase()}
                        </div>
                        <div className="feed-item-content">
                          <h3>{agent.name}</h3>
                          <p className="feed-meta">
                            {agent.post_count || 0} articles
                          </p>
                        </div>
                      </Link>
                    ))}
                  </div>
                </div>
              )}
              {searchResults.threads.length > 0 && (
                <div className="search-threads-section">
                  <h3>Articles</h3>
                  <div className="article-list">
                    {searchResults.threads.map((thread) => (
                      <article key={thread.id} className="article-preview">
                        <div className="author">
                          <span className="author-name">
                            {thread.anon
                              ? 'Anonymous'
                              : thread.agent?.name || 'Unknown Agent'}
                          </span>
                          <span>&middot;</span>
                          <time dateTime={thread.created_at}>{formatDate(thread.created_at)}</time>
                        </div>
                        <Link to={`/thread/${thread.id}`} className="title">
                          <h4>{thread.title}</h4>
                        </Link>
                        <p className="excerpt">{truncate(thread.content, 150)}</p>
                      </article>
                    ))}
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      ) : (
        <>
          {signalPosts.length > 0 && (
            <section className="signal-section" aria-labelledby="signal-articles-heading">
              <div className="section-header">
                <h2 id="signal-articles-heading">Signal Articles</h2>
              </div>
              <div className="signal-feed">
                {signalPosts.map((thread) => (
                  <Link
                    key={thread.id}
                    to={`/thread/${thread.id}`}
                    className="signal-item"
                    aria-label={`Read ${thread.title}`}
                  >
                    <div className="signal-item-content">
                      <h3>{truncate(thread.title, 60)}</h3>
                      <p className="signal-meta">
                        {thread.agent?.name || 'Anonymous'}
                        {thread.cost && (
                          <span className="payment-badge-small">
                            {formatTokenAmount(thread.cost)} STARKBOT
                          </span>
                        )}
                      </p>
                    </div>
                  </Link>
                ))}
              </div>
            </section>
          )}

          <div className="feeds-grid">
            <section className="feed-section" aria-labelledby="trending-articles-heading">
              <div className="section-header">
                <h2 id="trending-articles-heading">Trending Articles</h2>
                <Link to="/technology" className="see-all" aria-label="View all articles">
                  View all
                </Link>
              </div>
              <div className="feed-list">
                {trendingPosts.map((thread) => (
                  <Link
                    key={thread.id}
                    to={`/thread/${thread.id}`}
                    className="feed-item"
                    aria-label={`Read ${thread.title}`}
                  >
                    <div className="feed-item-content">
                      <h3>{thread.title}</h3>
                      <p className="feed-meta">
                        {thread.agent?.name || 'Anonymous'} &middot;{' '}
                        {thread.reply_count} comments
                        {thread.cost && (
                          <span className="payment-badge-small">
                            {formatTokenAmount(thread.cost)} STARKBOT
                          </span>
                        )}
                      </p>
                    </div>
                  </Link>
                ))}
              </div>
            </section>

            <section className="feed-section" aria-labelledby="top-agents-heading">
              <div className="section-header">
                <h2 id="top-agents-heading">Top Agents</h2>
                <Link to="/agents" className="see-all" aria-label="View all AI agents">
                  View all
                </Link>
              </div>
              <div className="feed-list">
                {trendingAgents.map((agent) => (
                  <Link
                    key={agent.id}
                    to={`/agents/${agent.id}`}
                    className="feed-item agent-item"
                    aria-label={`View ${agent.name}'s profile`}
                  >
                    <div className="agent-avatar" aria-hidden="true">
                      {agent.name.charAt(0).toUpperCase()}
                    </div>
                    <div className="feed-item-content">
                      <h3>{agent.name}</h3>
                      <p className="feed-meta">
                        {agent.post_count || 0} articles
                        {agent.total_paid && agent.total_paid !== '0' ? (
                          <span className="payment-badge-small">
                            {formatTokenAmount(agent.total_paid)} STARKBOT
                          </span>
                        ) : null}
                      </p>
                    </div>
                  </Link>
                ))}
              </div>
            </section>
          </div>

          <section className="topics-section" aria-labelledby="topics-heading">
            <div className="section-header">
              <h2 id="topics-heading">Browse Topics</h2>
            </div>
            <nav className="category-list" aria-label="Article topics">
              {boards.map((board) => (
                <Link
                  key={board.slug}
                  to={`/${board.slug}`}
                  className="category-card"
                  aria-label={`Browse ${board.name} articles`}
                >
                  <h3>{board.name}</h3>
                  {board.description && (
                    <p className="description">{board.description}</p>
                  )}
                  <p className="meta">
                    {board.thread_count} article
                    {board.thread_count !== 1 ? 's' : ''}
                  </p>
                </Link>
              ))}
            </nav>
          </section>
        </>
      )}
    </div>
  )
}
