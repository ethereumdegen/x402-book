interface PostContentProps {
  content: string
}

export default function PostContent({ content }: PostContentProps) {
  // Parse content and add greentext/quote styling
  const lines = content.split('\n')

  return (
    <div className="post-content">
      {lines.map((line, i) => {
        // Greentext (lines starting with >)
        if (line.startsWith('>') && !line.startsWith('>>')) {
          return (
            <span key={i}>
              <span className="greentext">{line}</span>
              {i < lines.length - 1 && '\n'}
            </span>
          )
        }

        // Quote links (>>uuid format)
        const quoteRegex = />>([a-f0-9-]{36})/g
        const parts: (string | JSX.Element)[] = []
        let lastIndex = 0
        let match

        while ((match = quoteRegex.exec(line)) !== null) {
          if (match.index > lastIndex) {
            parts.push(line.slice(lastIndex, match.index))
          }
          parts.push(
            <a key={`${i}-${match.index}`} href={`#${match[1]}`} className="quote">
              {match[0]}
            </a>
          )
          lastIndex = match.index + match[0].length
        }

        if (lastIndex < line.length) {
          parts.push(line.slice(lastIndex))
        }

        if (parts.length === 0) {
          parts.push(line)
        }

        return (
          <span key={i}>
            {parts}
            {i < lines.length - 1 && '\n'}
          </span>
        )
      })}
    </div>
  )
}
