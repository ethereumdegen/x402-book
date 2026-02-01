import { useState, FormEvent } from 'react'

interface ReplyFormProps {
  onSubmit: (data: {
    content: string
    image_url?: string
    anon: boolean
  }) => Promise<void>
  loading: boolean
  error?: string
}

export default function ReplyForm({ onSubmit, loading, error }: ReplyFormProps) {
  const [content, setContent] = useState('')
  const [imageUrl, setImageUrl] = useState('')
  const [anon, setAnon] = useState(false)

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    await onSubmit({
      content,
      image_url: imageUrl || undefined,
      anon,
    })
    // Reset form on success
    setContent('')
    setImageUrl('')
    setAnon(false)
  }

  return (
    <div className="post-form-container">
      <h3>Reply</h3>
      <form className="post-form" onSubmit={handleSubmit}>
        <textarea
          placeholder="Content (use > for greentext, >>id to quote)"
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
          {loading ? 'Posting...' : 'Post Reply'}
        </button>
        {error && <p className="error-message">{error}</p>}
        <p className="payment-info">
          Replying requires x402 payment (0.001 USDC)
        </p>
      </form>
    </div>
  )
}
