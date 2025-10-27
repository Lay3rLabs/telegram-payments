import { useState, useEffect } from 'react'
import WebApp from '@twa-dev/sdk'
import { AccountCreation } from './components/AccountCreation'
import { Dashboard } from './components/Dashboard'
import { WalletConnect } from './pages/WalletConnect'
import { hasAccount } from './services/storage'
import './App.css'

function App() {
  const [accountExists, setAccountExists] = useState<boolean>(false)
  const [isReady, setIsReady] = useState(false)
  const [currentPath, setCurrentPath] = useState(window.location.pathname)

  useEffect(() => {
    // Initialize Telegram Web App
    WebApp.ready()
    WebApp.expand()

    // Set header color to match theme
    WebApp.setHeaderColor('bg_color')

    // Check if account exists
    setAccountExists(hasAccount())
    setIsReady(true)

    // Log Telegram user info (if available)
    if (WebApp.initDataUnsafe.user) {
      console.log('Telegram User:', WebApp.initDataUnsafe.user)
    }

    // Listen for path changes
    const handlePathChange = () => {
      setCurrentPath(window.location.pathname)
    }
    window.addEventListener('popstate', handlePathChange)
    return () => window.removeEventListener('popstate', handlePathChange)
  }, [])

  const handleAccountCreated = () => {
    setAccountExists(true)
  }

  const handleLogout = () => {
    setAccountExists(false)
  }

  if (!isReady) {
    return (
      <div className="loading">
        <div className="spinner"></div>
        <p>Loading...</p>
      </div>
    )
  }

  // Route for wallet connection redirect
  if (currentPath === '/connect') {
    return <WalletConnect />
  }

  // Main app routes
  return (
    <div className="app">
      {accountExists ? (
        <Dashboard onLogout={handleLogout} />
      ) : (
        <AccountCreation onAccountCreated={handleAccountCreated} />
      )}
    </div>
  )
}

export default App
