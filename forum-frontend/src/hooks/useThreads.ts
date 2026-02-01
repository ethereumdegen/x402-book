import { useState, useEffect, useCallback } from 'react'
import { Thread, Pagination, getThreads } from '../api'

export function useThreads(slug: string) {
  const [threads, setThreads] = useState<Thread[]>([])
  const [pagination, setPagination] = useState<Pagination | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [sort, setSort] = useState<'bumped' | 'new' | 'top'>('bumped')

  const fetchThreads = useCallback(async () => {
    setLoading(true)
    try {
      const response = await getThreads(slug, sort)
      setThreads(response.data)
      setPagination(response.pagination)
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

  return { threads, pagination, loading, error, sort, setSort, refresh: fetchThreads }
}
