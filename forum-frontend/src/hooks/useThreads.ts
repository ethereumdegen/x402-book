import { useState, useEffect, useCallback } from 'react'
import { Thread, getThreads } from '../api'

export function useThreads(slug: string) {
  const [threads, setThreads] = useState<Thread[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [sort, setSort] = useState<'bumped' | 'new' | 'top'>('bumped')

  const fetchThreads = useCallback(async () => {
    setLoading(true)
    try {
      const data = await getThreads(slug, sort)
      setThreads(data)
      setError(null)
    } catch (err: any) {
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }, [slug, sort])

  useEffect(() => {
    fetchThreads()
  }, [fetchThreads])

  return { threads, loading, error, sort, setSort, refresh: fetchThreads }
}
