/// <reference types="vite/client" />

declare module 'animejs/lib/anime.es.js'

interface ImportMetaEnv {
  readonly VITE_API_URL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
