import { type ReactNode, useEffect, useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import DocsSidenav from './DocsSidenav'
import docsConfig from '../../config/docs-config'

interface Props {
  title: string
  children: ReactNode
}

interface TocItem {
  id: string
  text: string
  level: number
}

export default function DocsWrapper({ title, children }: Props) {
  const location = useLocation()
  const [toc, setToc] = useState<TocItem[]>([])
  const [activeId, setActiveId] = useState<string>('')

  // Find the current section for breadcrumb
  const currentSection = docsConfig.sections.find(section =>
    section.items.some(item => item.to === location.pathname)
  )

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
      {/* Left Sidebar */}
      <aside className="docs-sidebar">
        <Link to="/" className="docs-logo">
          x402 <span>Book</span>
        </Link>
        <DocsSidenav />
      </aside>

      {/* Main Content */}
      <main className="docs-main">
        <div className="docs-header">
          {currentSection && (
            <div className="docs-breadcrumb">{currentSection.title}</div>
          )}
          <h1>{title}</h1>
        </div>
        <div className="docs-content">
          {children}
        </div>
      </main>

      {/* Right Sidebar - Table of Contents */}
      {toc.length > 0 && (
        <aside className="docs-toc">
          <h4>On this page</h4>
          <nav>
            {toc.map((item) => (
              <button
                key={item.id}
                onClick={() => scrollToHeading(item.id)}
                className={`toc-link ${item.level === 3 ? 'indent' : ''} ${
                  activeId === item.id ? 'active' : ''
                }`}
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
