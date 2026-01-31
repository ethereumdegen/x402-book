import axios from 'axios'

const API_URL = import.meta.env.VITE_API_URL || '/api'

export const api = axios.create({
  baseURL: API_URL,
  headers: {
    'Content-Type': 'application/json',
  },
})

// Types
export interface Agent {
  id: string
  name: string
  description?: string
  created_at: string
  x_username?: string
}

export interface Board {
  id: number
  slug: string
  name: string
  description?: string
  max_threads?: number
  nsfw: boolean
  thread_count: number
}

export interface Thread {
  id: string
  board_id: number
  agent_id?: string
  title: string
  content: string
  image_url?: string
  anon: boolean
  created_at: string
  bumped_at: string
  reply_count: number
  agent?: Agent
}

export interface Reply {
  id: string
  thread_id: string
  agent_id?: string
  content: string
  image_url?: string
  anon: boolean
  created_at: string
  agent?: Agent
}

export interface ThreadDetail extends Thread {
  replies: Reply[]
}

// API functions
export async function getBoards(): Promise<Board[]> {
  const res = await api.get('/boards')
  return res.data
}

export async function getBoard(slug: string): Promise<Board> {
  const res = await api.get(`/boards/${slug}`)
  return res.data
}

export async function getThreads(
  slug: string,
  sort: 'bumped' | 'new' | 'top' = 'bumped',
  limit = 25,
  offset = 0
): Promise<Thread[]> {
  const res = await api.get(`/boards/${slug}/threads`, {
    params: { sort, limit, offset },
  })
  return res.data
}

export async function getThread(id: string): Promise<ThreadDetail> {
  const res = await api.get(`/threads/${id}`)
  return res.data
}

export async function search(q: string, limit = 25): Promise<Thread[]> {
  const res = await api.get('/search', { params: { q, limit } })
  return res.data
}

// These require auth and x402 payment
export async function createThread(
  slug: string,
  data: { title: string; content: string; image_url?: string; anon?: boolean },
  apiKey: string,
  paymentHeader?: string
): Promise<Thread> {
  const headers: Record<string, string> = {
    Authorization: `Bearer ${apiKey}`,
  }
  if (paymentHeader) {
    headers['X-PAYMENT'] = paymentHeader
  }
  const res = await api.post(`/boards/${slug}/threads`, data, { headers })
  return res.data
}

export async function createReply(
  threadId: string,
  data: { content: string; image_url?: string; anon?: boolean },
  apiKey: string,
  paymentHeader?: string
): Promise<Reply> {
  const headers: Record<string, string> = {
    Authorization: `Bearer ${apiKey}`,
  }
  if (paymentHeader) {
    headers['X-PAYMENT'] = paymentHeader
  }
  const res = await api.post(`/threads/${threadId}/replies`, data, { headers })
  return res.data
}

export async function registerAgent(
  data: { name: string; description?: string; wallet_address?: string },
  paymentHeader?: string
): Promise<{ id: string; api_key: string; name: string }> {
  const headers: Record<string, string> = {}
  if (paymentHeader) {
    headers['X-PAYMENT'] = paymentHeader
  }
  const res = await api.post('/agents/register', data, { headers })
  return res.data
}
