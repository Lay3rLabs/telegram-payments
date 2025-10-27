import { useState, useEffect } from "react";
import WebApp from "@twa-dev/sdk";
import { getStoredAccount, clearAccount } from "../services/storage";
import type { StoredAccount } from "../services/storage";
import { getNeutronBalance, type NeutronBalance } from "../services/neutron";
import {
  showAlert,
  showConfirm,
  hapticImpact,
  hapticNotification,
} from "../utils/telegram";
import "./Dashboard.css";

interface DashboardProps {
  onLogout: () => void;
}

interface TelegramUser {
  id: number;
  is_bot?: boolean;
  first_name?: string;
  last_name?: string;
  username?: string;
  language_code?: string;
}

export function Dashboard({ onLogout }: DashboardProps) {
  const [account, setAccount] = useState<StoredAccount | null>(null);
  const [balance, setBalance] = useState<NeutronBalance | null>(null);
  const [loadingBalance, setLoadingBalance] = useState(true);
  const [copied, setCopied] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const [telegramUser, setTelegramUser] = useState<TelegramUser | null>(null);

  useEffect(() => {
    const storedAccount = getStoredAccount();
    setAccount(storedAccount);

    // Get Telegram user info
    const user = WebApp.initDataUnsafe.user;
    setTelegramUser(user ?? null);

    // Fetch Neutron balance
    if (storedAccount) {
      fetchBalance(storedAccount.address);
    }

    // Setup Settings Button
    WebApp.SettingsButton.show();
    WebApp.SettingsButton.onClick(handleSettingsClick);

    return () => {
      WebApp.SettingsButton.hide();
      WebApp.SettingsButton.offClick(handleSettingsClick);
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

  const handleSettingsClick = () => {
    hapticImpact("light");
    setShowMenu(true);
  };

  const handleRefreshBalance = () => {
    if (account) {
      hapticImpact("light");
      fetchBalance(account.address);
    }
  };

  const handleLogout = () => {
    showConfirm(
      "Are you sure you want to remove this account? Make sure you have your private key saved!",
      (confirmed) => {
        if (confirmed) {
          hapticNotification("success");
          clearAccount();
          onLogout();
        } else {
          hapticImpact("light");
        }
      }
    );
  };

  const copyAddress = async () => {
    if (!account) return;
    try {
      await navigator.clipboard.writeText(account.address);
      hapticNotification("success");
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
      hapticNotification("error");
      showAlert("Failed to copy to clipboard");
    }
  };

  if (!account) {
    return <div>Loading...</div>;
  }

  return (
    <div className="dashboard">
      <div className="dashboard-card">
        <h1>Neutron Wallet</h1>

        <div className="account-info">
          {telegramUser && (
            <div className="info-section">
              <label>Telegram ID</label>
              <div className="info-value">
                {telegramUser.id}
                {telegramUser.username && (
                  <span
                    style={{
                      marginLeft: "8px",
                      color: "var(--tg-theme-hint-color, #999)",
                    }}
                  >
                    @{telegramUser.username}
                  </span>
                )}
              </div>
            </div>
          )}

          <div className="info-section">
            <label>Neutron Balance</label>
            <div className="balance-display">
              {loadingBalance ? (
                <span>Loading...</span>
              ) : (
                <span className="balance-amount">
                  {balance?.formatted || "0 NTRN"}
                </span>
              )}
              <button
                className="refresh-button"
                onClick={handleRefreshBalance}
                disabled={loadingBalance}
              >
                🔄
              </button>
            </div>
          </div>

          <div className="info-section">
            <label>Neutron Address</label>
            <div className="address-display">
              <code>{account.address}</code>
              <button className="copy-button" onClick={copyAddress}>
                {copied ? "✓" : "Copy"}
              </button>
            </div>
          </div>
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
          <div
            className="settings-content"
            onClick={(e) => e.stopPropagation()}
          >
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
