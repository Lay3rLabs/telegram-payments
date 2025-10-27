import { useState, useEffect } from 'react';
import WebApp from '@twa-dev/sdk';
import { getStoredAccount, clearAccount } from '../services/storage';
import type { StoredAccount } from '../services/storage';
import { showAlert, showConfirm, hapticImpact, hapticNotification } from '../utils/telegram';
import './Dashboard.css';

interface DashboardProps {
  onLogout: () => void;
}

export function Dashboard({ onLogout }: DashboardProps) {
  const [account, setAccount] = useState<StoredAccount | null>(null);
  const [copied, setCopied] = useState(false);
  const [showMenu, setShowMenu] = useState(false);

  useEffect(() => {
    const storedAccount = getStoredAccount();
    setAccount(storedAccount);

    // Setup Settings Button
    WebApp.SettingsButton.show();
    WebApp.SettingsButton.onClick(handleSettingsClick);

    return () => {
      WebApp.SettingsButton.hide();
      WebApp.SettingsButton.offClick(handleSettingsClick);
    };
  }, []);

  const handleSettingsClick = () => {
    hapticImpact('light');
    setShowMenu(true);
  };

  const handleLogout = () => {
    showConfirm(
      'Are you sure you want to remove this account? Make sure you have your private key saved!',
      (confirmed) => {
        if (confirmed) {
          hapticNotification('success');
          clearAccount();
          onLogout();
        } else {
          hapticImpact('light');
        }
      }
    );
  };

  const copyAddress = async () => {
    if (!account) return;
    try {
      await navigator.clipboard.writeText(account.address);
      hapticNotification('success');
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
      hapticNotification('error');
      showAlert('Failed to copy to clipboard');
    }
  };

  if (!account) {
    return <div>Loading...</div>;
  }

  return (
    <div className="dashboard">
      <div className="dashboard-card">
        <h1>Your Account</h1>

        <div className="account-info">
          <div className="info-section">
            <label>Address</label>
            <div className="address-display">
              <code>{account.address}</code>
              <button className="copy-button" onClick={copyAddress}>
                {copied ? '✓' : 'Copy'}
              </button>
            </div>
          </div>

          <div className="info-section">
            <label>Created</label>
            <div className="created-date">
              {new Date(account.createdAt).toLocaleDateString()}
            </div>
          </div>
        </div>

        <div className="placeholder-section">
          <h3>Coming Soon</h3>
          <ul>
            <li>Send payments to Telegram users</li>
            <li>Receive payment notifications</li>
            <li>View transaction history</li>
            <li>Manage multiple assets</li>
          </ul>
        </div>

        <div className="action-buttons">
          <button className="action-button" onClick={handleLogout}>
            🗑️ Remove Account
          </button>
        </div>

        <p className="settings-hint">
          ⚙️ Use the settings button (top right) for more options
        </p>
      </div>

      {showMenu && (
        <div className="settings-menu" onClick={() => setShowMenu(false)}>
          <div className="settings-content" onClick={(e) => e.stopPropagation()}>
            <h3>Settings</h3>
            <button className="menu-item" onClick={handleLogout}>
              🗑️ Remove Account
            </button>
            <button className="menu-item" onClick={() => setShowMenu(false)}>
              ❌ Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
