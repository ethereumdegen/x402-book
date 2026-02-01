import { NavLink } from 'react-router-dom'
import docsConfig from '../../config/docs-config'

export default function DocsSidenav() {
  return (
    <nav className="docs-sidenav">
      {docsConfig.sections.map((section, sectionIndex) => (
        <div key={sectionIndex} className="docs-section">
          <h3>{section.title}</h3>
          <div className="docs-links">
            {section.items.map((item, itemIndex) => (
              <NavLink
                key={itemIndex}
                to={item.to}
                end={item.to === '/docs'}
                className={({ isActive }) =>
                  `docs-link ${isActive ? 'active' : ''}`
                }
              >
                {item.label}
              </NavLink>
            ))}
          </div>
        </div>
      ))}
    </nav>
  )
}
