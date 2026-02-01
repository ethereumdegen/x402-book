import { type ReactNode, useEffect, useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import DocsSidenav from './DocsSidenav'
import docsConfig from '../../config/docs-config'
import { SEO, BreadcrumbSchema, SITE_URL } from '../../components/SEO'

interface Props {
  title: string
  description?: string
  children: ReactNode
}

interface TocItem {
  id: string
  text: string
  level: number
}

export default function DocsWrapper({ title, description, children }: Props) {
  const location = useLocation()
  const [toc, setToc] = useState<TocItem[]>([])
  const [activeId, setActiveId] = useState<string>('')

  // Find the current section for breadcrumb
  const currentSection = docsConfig.sections.find(section =>
    section.items.some(item => item.to === location.pathname)
  )

  const pageUrl = `${SITE_URL}${location.pathname}`
  const pageDescription = description || `${title} - x402 Book API documentation. Learn how to integrate with the x402 payment protocol for AI agent publishing.`

  const breadcrumbs = [
    { name: 'Home', url: SITE_URL },
    { name: 'Documentation', url: `${SITE_URL}/docs` },
    ...(currentSection && location.pathname !== '/docs'
      ? [{ name: title, url: pageUrl }]
      : []),
  ]

  // Extract table of contents from rendered content
  useEffect(() => {
    const timer = setTimeout(() => {
      const headings = document.querySelectorAll('.docs-content h2, .docs-content h3')
      const items: TocItem[] = Array.from(headings).map((heading, index) => {
        const id = heading.id || `heading-${index}`
        if (!heading.id) {
          heading.id = id
        }
        return {
          id,
          text: heading.textContent || '',
          level: heading.tagName === 'H2' ? 2 : 3,
        }
      })
      setToc(items)
    }, 100)

    return () => clearTimeout(timer)
  }, [location.pathname, children])

  // Track active heading on scroll
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            setActiveId(entry.target.id)
          }
        })
      },
      { rootMargin: '-80px 0px -80% 0px' }
    )

    const headings = document.querySelectorAll('.docs-content h2, .docs-content h3')
    headings.forEach((heading) => observer.observe(heading))

    return () => observer.disconnect()
  }, [toc])

  const scrollToHeading = (id: string) => {
    const element = document.getElementById(id)
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }

  return (
    <div className="docs-layout">
      <SEO
        title={`${title} - Documentation`}
        description={pageDescription}
        url={pageUrl}
        type="website"
      />
      <BreadcrumbSchema items={breadcrumbs} />

      {/* Left Sidebar */}
      <aside className="docs-sidebar" aria-label="Documentation navigation">
        <Link to="/" className="docs-logo" aria-label="Go to x402 Book homepage">
          x402 <span>Book</span>
        </Link>
        <DocsSidenav />
      </aside>

      {/* Main Content */}
      <main className="docs-main" id="main-content">
        <div className="docs-header">
          {currentSection && (
            <nav className="docs-breadcrumb" aria-label="Breadcrumb">
              {currentSection.title}
            </nav>
          )}
          <h1>{title}</h1>
        </div>
        <article className="docs-content">
          {children}
        </article>
      </main>

      {/* Right Sidebar - Table of Contents */}
      {toc.length > 0 && (
        <aside className="docs-toc" aria-label="Table of contents">
          <h4 id="toc-heading">On this page</h4>
          <nav aria-labelledby="toc-heading">
            {toc.map((item) => (
              <button
                key={item.id}
                onClick={() => scrollToHeading(item.id)}
                className={`toc-link ${item.level === 3 ? 'indent' : ''} ${
                  activeId === item.id ? 'active' : ''
                }`}
                aria-current={activeId === item.id ? 'location' : undefined}
              >
                {item.text}
              </button>
            ))}
          </nav>
        </aside>
      )}
    </div>
  )
}
