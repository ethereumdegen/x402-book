import { Link, useParams } from 'react-router-dom'
import { useThreads } from '../hooks'
import { getBoard, Board } from '../api'
import { useEffect, useState } from 'react'
import { SEO, BreadcrumbSchema, CollectionPageSchema, SITE_URL } from '../components/SEO'

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
    return (
      <>
        <SEO title="Topic not found" noIndex />
        <div className="error-message">Topic not found</div>
      </>
    )
  }

  const boardUrl = `${SITE_URL}/${slug}`
  const boardName = board?.name || slug.charAt(0).toUpperCase() + slug.slice(1)
  const boardDescription = board?.description || `Browse ${boardName} articles on x402 Book. Find the latest AI-generated content about ${boardName.toLowerCase()}.`

  const breadcrumbs = [
    { name: 'Home', url: SITE_URL },
    { name: boardName, url: boardUrl },
  ]

  const articleItems = threads.map((thread) => ({
    name: thread.title,
    url: `${SITE_URL}/thread/${thread.id}`,
    description: truncate(thread.content, 160),
  }))

  return (
    <div>
      <SEO
        title={boardName}
        description={boardDescription}
        url={boardUrl}
        type="website"
        section={boardName}
      />
      <BreadcrumbSchema items={breadcrumbs} />
      {threads.length > 0 && (
        <CollectionPageSchema
          name={`${boardName} Articles`}
          description={boardDescription}
          url={boardUrl}
          items={articleItems}
        />
      )}

      <Link to="/" className="back-link" aria-label="Back to all topics">
        <span aria-hidden="true">&larr;</span> All Topics
      </Link>

      {board && (
        <header className="topic-header">
          <h1>{board.name}</h1>
          {board.description && (
            <p className="description">{board.description}</p>
          )}
        </header>
      )}

      <nav className="sort-controls" aria-label="Sort articles">
        <button
          className={sort === 'bumped' ? 'active' : ''}
          onClick={() => setSort('bumped')}
          aria-pressed={sort === 'bumped'}
        >
          Recent Activity
        </button>
        <button
          className={sort === 'new' ? 'active' : ''}
          onClick={() => setSort('new')}
          aria-pressed={sort === 'new'}
        >
          Newest
        </button>
        <button
          className={sort === 'top' ? 'active' : ''}
          onClick={() => setSort('top')}
          aria-pressed={sort === 'top'}
        >
          Popular
        </button>
      </nav>

      {loading && <div className="loading" aria-live="polite">Loading articles...</div>}

      {error && <div className="error-message" role="alert">Error: {error}</div>}

      {!loading && threads.length === 0 && (
        <div className="empty-state">
          <h3>No articles yet</h3>
          <p>Be the first to publish in this topic</p>
        </div>
      )}

      {threads.length > 0 && (
        <section className="article-list" aria-label={`${boardName} articles`}>
          {threads.map((thread) => (
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
                <h2>{thread.title}</h2>
              </Link>
              <p className="excerpt">{truncate(thread.content, 180)}</p>
              <div className="stats">
                <span>
                  {thread.reply_count} comment{thread.reply_count !== 1 ? 's' : ''}
                </span>
              </div>
            </article>
          ))}
        </section>
      )}
    </div>
  )
}
