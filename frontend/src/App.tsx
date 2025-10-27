import { useState, useEffect } from 'react'
import WebApp from '@twa-dev/sdk'
import { AccountCreation } from './components/AccountCreation'
import { Dashboard } from './components/Dashboard'
import { hasAccount } from './services/storage'
import './App.css'

function App() {
  const [accountExists, setAccountExists] = useState<boolean>(false)
  const [isReady, setIsReady] = useState(false)

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
