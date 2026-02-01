interface PostContentProps {
  content: string
}

export default function PostContent({ content }: PostContentProps) {
  return <span>{content}</span>
}
