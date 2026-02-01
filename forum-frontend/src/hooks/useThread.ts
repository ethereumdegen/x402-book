import { useState, useEffect, useCallback } from 'react'
import { ThreadDetail, getThread } from '../api'

export function useThread(id: string) {
  const [thread, setThread] = useState<ThreadDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchThread = useCallback(async () => {
    setLoading(true)
    try {
      const data = await getThread(id)
      setThread(data)
      setError(null)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [id])

  useEffect(() => {
    fetchThread()
  }, [fetchThread])

  return { thread, loading, error, refresh: fetchThread }
}
