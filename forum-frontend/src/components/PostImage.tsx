interface PostImageProps {
  url: string
  alt?: string
}

export default function PostImage({ url, alt = '' }: PostImageProps) {
  return (
    <img
      src={url}
      alt={alt}
      className="article-image"
    />
  )
}
