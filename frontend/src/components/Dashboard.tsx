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
import { useSigningClient } from "../hooks/useSigningClient";
import { TgContractPaymentsClient } from "../contracts/TgContractPayments.client";
import { WalletManager } from "./WalletManager";
import { getContractAddress, setContractAddress } from "../config/contracts";

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
  const [contractClient, setContractClient] =
    useState<TgContractPaymentsClient | null>(null);
  const [contractAddress, setContractAddressState] = useState<string>("");
  const [showContractSetup, setShowContractSetup] = useState(false);
  const [contractAddressInput, setContractAddressInput] = useState("");

  // Use the signing client hook
  const {
    client: signingClient,
    address: signerAddress,
    walletType,
    isReady: isSignerReady,
  } = useSigningClient();

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

    // Try to load contract address
    try {
      const addr = getContractAddress("payments");
      setContractAddressState(addr);
    } catch (err) {
      // Contract address not configured yet
      console.log("Contract address not configured:", err);
    }

    // Setup Settings Button
    WebApp.SettingsButton.show();
    WebApp.SettingsButton.onClick(handleSettingsClick);

    return () => {
      WebApp.SettingsButton.hide();
      WebApp.SettingsButton.offClick(handleSettingsClick);
    };
  }, []);

  // Initialize contract client when signing client is ready
  useEffect(() => {
    if (
      signingClient &&
      signerAddress &&
      contractAddress &&
      contractAddress !== "neutron1..."
    ) {
      try {
        const client = new TgContractPaymentsClient(
          signingClient,
          signerAddress,
          contractAddress
        );
        setContractClient(client);
        console.log("✅ Contract client initialized:", contractAddress);
      } catch (err) {
        console.error("Failed to initialize contract client:", err);
      }
    } else {
      setContractClient(null);
    }
  }, [signingClient, signerAddress, contractAddress]);

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

  const handleSetContractAddress = () => {
    if (!contractAddressInput.trim()) {
      showAlert("Please enter a contract address");
      return;
    }

    if (!contractAddressInput.startsWith("neutron1")) {
      showAlert('Contract address must start with "neutron1"');
      return;
    }

    setContractAddress("payments", contractAddressInput.trim());
    setContractAddressState(contractAddressInput.trim());
    setShowContractSetup(false);
    setContractAddressInput("");
    hapticNotification("success");
    showAlert("Contract address saved!");
  };

  if (!account) {
    return <div>Loading...</div>;
  }

  return (
    <div className="dashboard">
      <div className="dashboard-card">
        <h1>Telegram Payments</h1>

        {/* Wallet Manager - shows initialization UI if needed */}
        <WalletManager />

        {/* Signer Status */}
        {isSignerReady && signingClient && (
          <div
            style={{
              padding: "12px",
              background: "rgba(40, 167, 69, 0.1)",
              borderRadius: "8px",
              marginBottom: "16px",
              border: "1px solid rgba(40, 167, 69, 0.3)",
            }}
          >
            <div
              style={{ fontSize: "13px", color: "#28a745", fontWeight: 600 }}
            >
              ✅ Signer Ready ({walletType})
            </div>
          </div>
        )}

        {/* Contract Status */}
        <div
          style={{
            padding: "12px",
            background: contractClient
              ? "rgba(40, 167, 69, 0.1)"
              : "rgba(255, 193, 7, 0.1)",
            borderRadius: "8px",
            marginBottom: "16px",
            border: contractClient
              ? "1px solid rgba(40, 167, 69, 0.3)"
              : "1px solid rgba(255, 193, 7, 0.3)",
          }}
        >
          <div
            style={{
              fontSize: "13px",
              color: contractClient ? "#28a745" : "#856404",
              fontWeight: 600,
              marginBottom: "4px",
            }}
          >
            {contractClient
              ? "✅ Contract Client Ready"
              : "⚠️ Contract Not Configured"}
          </div>
          {contractAddress && contractAddress !== "neutron1..." ? (
            <div
              style={{
                fontSize: "11px",
                color: "var(--tg-theme-hint-color, #666)",
                fontFamily: "monospace",
                wordBreak: "break-all",
              }}
            >
              {contractAddress}
            </div>
          ) : (
            <button
              onClick={() => setShowContractSetup(!showContractSetup)}
              style={{
                marginTop: "8px",
                padding: "6px 12px",
                background: "var(--tg-theme-button-color, #3390ec)",
                color: "var(--tg-theme-button-text-color, #fff)",
                border: "none",
                borderRadius: "6px",
                fontSize: "12px",
                cursor: "pointer",
              }}
            >
              {showContractSetup ? "Cancel" : "Set Contract Address"}
            </button>
          )}
        </div>

        {/* Contract Setup */}
        {showContractSetup && (
          <div
            style={{
              padding: "16px",
              background: "var(--tg-theme-secondary-bg-color, #f0f0f0)",
              borderRadius: "8px",
              marginBottom: "16px",
            }}
          >
            <h3 style={{ margin: "0 0 12px 0", fontSize: "14px" }}>
              Configure Contract Address
            </h3>
            <input
              type="text"
              value={contractAddressInput}
              onChange={(e) => setContractAddressInput(e.target.value)}
              placeholder="neutron1..."
              style={{
                width: "100%",
                padding: "8px",
                borderRadius: "6px",
                border: "1px solid var(--tg-theme-hint-color, #ccc)",
                fontSize: "12px",
                fontFamily: "monospace",
                marginBottom: "8px",
              }}
            />
            <button
              onClick={handleSetContractAddress}
              style={{
                width: "100%",
                padding: "10px",
                background: "var(--tg-theme-button-color, #3390ec)",
                color: "var(--tg-theme-button-text-color, #fff)",
                border: "none",
                borderRadius: "6px",
                fontSize: "14px",
                fontWeight: 600,
                cursor: "pointer",
              }}
            >
              Save Contract Address
            </button>
          </div>
        )}

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
