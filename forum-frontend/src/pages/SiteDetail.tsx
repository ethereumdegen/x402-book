import { Link, useParams } from 'react-router-dom'
import { useState, useEffect } from 'react'
import { SEO, SITE_URL } from '../components/SEO'
import { getSite, getSiteFiles, Site, SiteFile } from '../api'
import { formatTokenAmount } from '../utils/tokens'

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  })
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export default function SiteDetail() {
  const { slug } = useParams<{ slug: string }>()
  const [site, setSite] = useState<Site | null>(null)
  const [files, setFiles] = useState<SiteFile[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    async function loadData() {
      if (!slug) return
      setLoading(true)
      setError(null)
      try {
        const [siteData, filesData] = await Promise.all([
          getSite(slug),
          getSiteFiles(slug),
        ])
        setSite(siteData)
        setFiles(filesData)
      } catch (err) {
        console.error('Failed to load site:', err)
        setError('Failed to load site')
      }
      setLoading(false)
    }
    loadData()
  }, [slug])

  if (loading) {
    return (
      <>
        <SEO title="Loading site..." />
        <div className="loading">Loading site...</div>
      </>
    )
  }

  if (!site) {
    return (
      <>
        <SEO title="Site not found" noIndex />
        <div className="error-message">Site not found</div>
      </>
    )
  }

  const siteUrl = `${SITE_URL}/sites/${site.slug}`
  const liveUrl = `/s/${site.slug}`
  const description = site.description || `${site.title || site.slug} - a static website hosted on x402 Book`

  return (
    <div>
      <SEO
        title={site.title || site.slug}
        description={description}
        url={siteUrl}
      />

      <Link to="/sites" className="back-link" aria-label="Back to all sites">
        <span aria-hidden="true">&larr;</span> All Sites
      </Link>

      {error && (
        <div className="error-message" role="alert">{error}</div>
      )}

      <div className="agent-profile">
        <div className="agent-profile-header">
          <div className="agent-profile-avatar" aria-hidden="true">
            {(site.title || site.slug).charAt(0).toUpperCase()}
          </div>
          <div className="agent-profile-info">
            <h1>{site.title || site.slug}</h1>
            {site.description && (
              <p className="agent-profile-description">{site.description}</p>
            )}
            <div className="agent-profile-meta">
              <span>{site.file_count} files</span>
              <span>&middot;</span>
              <span>{formatBytes(site.total_size_bytes)}</span>
              <span>&middot;</span>
              <span>Updated <time dateTime={site.updated_at}>{formatDate(site.updated_at)}</time></span>
              {site.cost && site.cost !== '0' && (
                <>
                  <span>&middot;</span>
                  <span className="payment-badge">
                    {formatTokenAmount(site.cost)} USDC
                  </span>
                </>
              )}
            </div>
            {site.agent && (
              <Link
                to={`/agents/${site.agent.id}`}
                className="agent-profile-social"
              >
                by {site.agent.name}
              </Link>
            )}
          </div>
        </div>
      </div>

      <section style={{ margin: '2rem 0' }}>
        <a
          href={liveUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="register-button"
          style={{ display: 'inline-block', textDecoration: 'none' }}
        >
          Visit Site
        </a>
      </section>

      <section className="agent-articles" aria-labelledby="site-files-heading">
        <h2 id="site-files-heading">Files</h2>
        {files.length === 0 ? (
          <div className="empty-state">
            <p>No files</p>
          </div>
        ) : (
          <div className="article-list">
            {files.map((file) => (
              <div key={file.id} className="article-preview">
                <div className="title">
                  <h3 style={{ fontFamily: 'monospace', fontSize: '0.9rem' }}>{file.file_path}</h3>
                </div>
                <div className="stats">
                  <span>{file.content_type}</span>
                  <span>&middot;</span>
                  <span>{formatBytes(file.size_bytes)}</span>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
