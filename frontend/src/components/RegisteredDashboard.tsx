import { useState, useEffect } from 'react';
import WebApp from '@twa-dev/sdk';
import { getStoredAccount, clearAccount } from '../services/storage';
import type { StoredAccount } from '../services/storage';
import { getNeutronBalance, type NeutronBalance } from '../services/neutron';
import {
  showAlert,
  showConfirm,
  hapticImpact,
  hapticNotification,
} from '../utils/telegram';
import { useSigningClient } from '../hooks/useSigningClient';
import { updateAuthzLimit, revokeAuthz } from '../services/authz';
import { getContractAddress } from '../config/contracts';
import './RegisteredDashboard.css';

interface RegisteredDashboardProps {
  onLogout: () => void;
}

export function RegisteredDashboard({ onLogout }: RegisteredDashboardProps) {
  const [account, setAccount] = useState<StoredAccount | null>(null);
  const [balance, setBalance] = useState<NeutronBalance | null>(null);
  const [loadingBalance, setLoadingBalance] = useState(true);
  const [copied, setCopied] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [telegramUser, setTelegramUser] = useState<any>(null);
  const [showLimitEditor, setShowLimitEditor] = useState(false);
  const [newLimit, setNewLimit] = useState('100');
  const [isUpdatingLimit, setIsUpdatingLimit] = useState(false);

  const { client: signingClient, address: signerAddress } = useSigningClient();

  useEffect(() => {
    const storedAccount = getStoredAccount();
    setAccount(storedAccount);

    // Get Telegram user info
    const user = WebApp.initDataUnsafe.user;
    setTelegramUser(user ?? null);

    // Fetch balance
    if (storedAccount) {
      fetchBalance(storedAccount.address);
    }

    // Setup Settings Button
    WebApp.SettingsButton.show();
    WebApp.SettingsButton.onClick(() => {
      hapticImpact('light');
      setShowSettings(true);
    });

    return () => {
      WebApp.SettingsButton.hide();
    };
  }, []);

  const fetchBalance = async (address: string) => {
    setLoadingBalance(true);
    try {
      const neutronBalance = await getNeutronBalance(address);
      setBalance(neutronBalance);
    } finally {
      setLoadingBalance(false);
    }
  };

  const copyAddress = async () => {
    if (!account) return;
    try {
      await navigator.clipboard.writeText(account.address);
      hapticNotification('success');
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      hapticNotification('error');
      showAlert('Failed to copy to clipboard');
    }
  };

  const handleUpdateLimit = async () => {
    if (!signingClient || !signerAddress) {
      showAlert('Wallet not initialized');
      return;
    }

    if (!newLimit || parseFloat(newLimit) <= 0) {
      showAlert('Please enter a valid limit');
      return;
    }

    let contractAddress: string;
    try {
      contractAddress = getContractAddress('payments');
    } catch (err) {
      showAlert('Contract address not configured');
      return;
    }

    setIsUpdatingLimit(true);
    hapticImpact('medium');

    try {
      await updateAuthzLimit(signingClient, {
        userAddress: signerAddress,
        contractAddress,
        newLimit,
      });

      hapticNotification('success');
      showAlert(`Spending limit updated to ${newLimit} NTRN`);
      setShowLimitEditor(false);
    } catch (err) {
      console.error('Failed to update limit:', err);
      hapticNotification('error');
      showAlert(`Failed to update limit: ${(err as Error).message}`);
    } finally {
      setIsUpdatingLimit(false);
    }
  };

  const handleRevokeAccess = () => {
    showConfirm(
      'Are you sure you want to revoke authorization? You will need to re-register to send payments via Telegram.',
      async (confirmed) => {
        if (!confirmed) return;

        if (!signingClient || !signerAddress) {
          showAlert('Wallet not initialized');
          return;
        }

        let contractAddress: string;
        try {
          contractAddress = getContractAddress('payments');
        } catch (err) {
          showAlert('Contract address not configured');
          return;
        }

        hapticImpact('medium');

        try {
          await revokeAuthz(signingClient, {
            userAddress: signerAddress,
            contractAddress,
          });

          hapticNotification('success');
          showAlert('Authorization revoked successfully');
        } catch (err) {
          console.error('Failed to revoke:', err);
          hapticNotification('error');
          showAlert(`Failed to revoke: ${(err as Error).message}`);
        }
      }
    );
  };

  const handleLogout = () => {
    showConfirm(
      'Are you sure you want to remove this account? Make sure you have your recovery phrase saved!',
      (confirmed) => {
        if (confirmed) {
          hapticNotification('success');
          clearAccount();
          onLogout();
        }
      }
    );
  };

  if (!account) {
    return <div>Loading...</div>;
  }

  return (
    <div className="registered-dashboard">
      {/* Header */}
      <div className="dashboard-header">
        <h1>💸 Telegram Payments</h1>
        {telegramUser && (
          <div className="user-badge">
            @{telegramUser.username || `user_${telegramUser.id}`}
          </div>
        )}
      </div>

      {/* Balance Card */}
      <div className="balance-card">
        <div className="balance-label">Total Balance</div>
        <div className="balance-amount">
          {loadingBalance ? (
            <span className="loading">Loading...</span>
          ) : (
            <span>{balance?.formatted || '0 NTRN'}</span>
          )}
        </div>
        <button
          className="refresh-button"
          onClick={() => account && fetchBalance(account.address)}
          disabled={loadingBalance}
        >
          🔄 Refresh
        </button>
      </div>

      {/* Quick Actions */}
      <div className="quick-actions">
        <button className="action-card" onClick={() => setShowLimitEditor(true)}>
          <div className="action-icon">💰</div>
          <div className="action-label">Spending Limit</div>
          <div className="action-value">Configure</div>
        </button>

        <button className="action-card" onClick={copyAddress}>
          <div className="action-icon">📋</div>
          <div className="action-label">Wallet Address</div>
          <div className="action-value">{copied ? 'Copied!' : 'Copy'}</div>
        </button>
      </div>

      {/* Account Info */}
      <div className="info-section">
        <h3>Account Details</h3>
        <div className="info-item">
          <span className="info-label">Telegram Handle</span>
          <span className="info-value">
            @{telegramUser?.username || `user_${telegramUser?.id}`}
          </span>
        </div>
        <div className="info-item">
          <span className="info-label">Wallet Address</span>
          <span className="info-value mono">
            {account.address.substring(0, 12)}...
            {account.address.substring(account.address.length - 8)}
          </span>
        </div>
      </div>

      {/* Limit Editor Modal */}
      {showLimitEditor && (
        <div className="modal-overlay" onClick={() => setShowLimitEditor(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h2>Update Spending Limit</h2>
            <p className="modal-description">
              Set the maximum amount that can be spent through Telegram in a single transaction.
            </p>

            <div className="limit-input-group">
              <input
                type="number"
                min="1"
                step="1"
                value={newLimit}
                onChange={(e) => setNewLimit(e.target.value)}
                placeholder="100"
              />
              <span className="input-suffix">NTRN</span>
            </div>

            <div className="limit-presets">
              <button onClick={() => setNewLimit('10')}>10</button>
              <button onClick={() => setNewLimit('50')}>50</button>
              <button onClick={() => setNewLimit('100')}>100</button>
              <button onClick={() => setNewLimit('500')}>500</button>
            </div>

            <div className="modal-actions">
              <button
                className="secondary-button"
                onClick={() => setShowLimitEditor(false)}
              >
                Cancel
              </button>
              <button
                className="primary-button"
                onClick={handleUpdateLimit}
                disabled={isUpdatingLimit}
              >
                {isUpdatingLimit ? 'Updating...' : 'Update Limit'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Settings Modal */}
      {showSettings && (
        <div className="modal-overlay" onClick={() => setShowSettings(false)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <h2>Settings</h2>

            <div className="settings-menu">
              <button className="menu-item" onClick={handleRevokeAccess}>
                <span>🔒</span>
                <div>
                  <div className="menu-item-title">Revoke Authorization</div>
                  <div className="menu-item-desc">
                    Remove spending authorization from contract
                  </div>
                </div>
              </button>

              <button className="menu-item danger" onClick={handleLogout}>
                <span>🗑️</span>
                <div>
                  <div className="menu-item-title">Remove Account</div>
                  <div className="menu-item-desc">
                    Delete wallet from this device
                  </div>
                </div>
              </button>
            </div>

            <button
              className="secondary-button"
              onClick={() => setShowSettings(false)}
              style={{ marginTop: '16px', width: '100%' }}
            >
              Close
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
