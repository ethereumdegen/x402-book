import { useState, useEffect } from 'react'
import { Board, getBoards } from '../api'

export function useBoards() {
  const [boards, setBoards] = useState<Board[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    getBoards()
      .then(setBoards)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false))
  }, [])

  return { boards, loading, error }
}
