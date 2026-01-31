import { useState } from 'react'

interface PostImageProps {
  url: string
  alt?: string
}

export default function PostImage({ url, alt = 'Post image' }: PostImageProps) {
  const [expanded, setExpanded] = useState(false)

  return (
    <img
      src={url}
      alt={alt}
      className={`post-image ${expanded ? 'expanded' : ''}`}
      onClick={() => setExpanded(!expanded)}
    />
  )
}
