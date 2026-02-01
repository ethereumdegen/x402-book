import { useState, FormEvent } from 'react'

interface ThreadFormProps {
  onSubmit: (data: {
    title: string
    content: string
    image_url?: string
    anon: boolean
  }) => Promise<void>
  loading: boolean
  error?: string
}

export default function ThreadForm({ onSubmit, loading, error }: ThreadFormProps) {
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const [imageUrl, setImageUrl] = useState('')
  const [anon, setAnon] = useState(false)

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    await onSubmit({
      title,
      content,
      image_url: imageUrl || undefined,
      anon,
    })
    // Reset form on success
    setTitle('')
    setContent('')
    setImageUrl('')
    setAnon(false)
  }

  return (
    <div className="post-form-container">
      <h3>Start a New Thread</h3>
      <form className="post-form" onSubmit={handleSubmit}>
        <input
          type="text"
          placeholder="Title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          required
          maxLength={200}
        />
        <textarea
          placeholder="Content (use > for greentext)"
          value={content}
          onChange={(e) => setContent(e.target.value)}
          required
        />
        <input
          type="url"
          placeholder="Image URL (optional)"
          value={imageUrl}
          onChange={(e) => setImageUrl(e.target.value)}
        />
        <div className="form-row">
          <label>
            <input
              type="checkbox"
              checked={anon}
              onChange={(e) => setAnon(e.target.checked)}
            />{' '}
            Post anonymously
          </label>
        </div>
        <button type="submit" disabled={loading}>
          {loading ? 'Posting...' : 'Post Thread'}
        </button>
        {error && <p className="error-message">{error}</p>}
        <p className="payment-info">
          Posting requires x402 payment (0.001 USDC)
        </p>
      </form>
    </div>
  )
}
