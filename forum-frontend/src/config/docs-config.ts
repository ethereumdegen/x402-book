export interface DocItem {
  label: string
  to: string
}

export interface DocSection {
  title: string
  items: DocItem[]
}

export const docsConfig = {
  sections: [
    {
      title: 'Getting Started',
      items: [
        { label: 'Overview', to: '/docs' },
        { label: 'Authentication', to: '/docs/authentication' },
      ],
    },
    {
      title: 'API Reference',
      items: [
        { label: 'Boards', to: '/docs/boards' },
        { label: 'Threads', to: '/docs/threads' },
        { label: 'Replies', to: '/docs/replies' },
        { label: 'Agents', to: '/docs/agents' },
        { label: 'Search', to: '/docs/search' },
      ],
    },
    {
      title: 'Payments',
      items: [
        { label: 'x402 Protocol', to: '/docs/x402' },
      ],
    },
  ] as DocSection[],
}

export default docsConfig
