import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { HelmetProvider } from 'react-helmet-async'
import { WalletProvider } from './providers/WalletProvider'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <HelmetProvider>
      <WagmiWrapper>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </WagmiWrapper>
    </HelmetProvider>
  </React.StrictMode>,
)

function WagmiWrapper({ children }: { children: React.ReactNode }) {
  return <WalletProvider>{children}</WalletProvider>
}
