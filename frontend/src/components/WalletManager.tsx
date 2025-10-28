import { useState } from 'react';
import { useSigningClient } from '../hooks/useSigningClient';
import { showAlert, hapticNotification } from '../utils/telegram';
import { hasMnemonic } from '../services/storage';

interface WalletManagerProps {
  onClientReady?: (client: any, address: string) => void;
}

/**
 * Component to manage wallet initialization
 * Use this in your Dashboard to get a signing client
 */
export function WalletManager({ onClientReady }: WalletManagerProps) {
  const { client, address, walletType, isReady, error, initializeWithMnemonic, initializeWithKeplr, initializeWithWalletConnect } = useSigningClient();
  const [isInitializing, setIsInitializing] = useState(false);
  const [mnemonicInput, setMnemonicInput] = useState('');

  // Debug logging
  console.log('WalletManager state:', { client: !!client, address, walletType, isReady, error: error?.message });

  // Notify parent when client is ready
  if (client && address && onClientReady) {
    onClientReady(client, address);
  }

  const handleMnemonicSubmit = async () => {
    if (!mnemonicInput.trim()) {
      showAlert('Please enter your mnemonic');
      return;
    }

    setIsInitializing(true);
    try {
      await initializeWithMnemonic(mnemonicInput.trim());
      hapticNotification('success');
      setMnemonicInput(''); // Clear input
    } catch (err) {
      hapticNotification('error');
      showAlert('Failed to initialize wallet: ' + (err as Error).message);
    } finally {
      setIsInitializing(false);
    }
  };

  const handleKeplrConnect = async () => {
    setIsInitializing(true);
    try {
      await initializeWithKeplr();
      hapticNotification('success');
    } catch (err) {
      hapticNotification('error');
      showAlert('Failed to connect to Keplr: ' + (err as Error).message);
    } finally {
      setIsInitializing(false);
    }
  };

  const handleWalletConnectConnect = async () => {
    setIsInitializing(true);
    try {
      await initializeWithWalletConnect();
      hapticNotification('success');
    } catch (err) {
      hapticNotification('error');
      showAlert('Failed to connect with WalletConnect: ' + (err as Error).message);
    } finally {
      setIsInitializing(false);
    }
  };

  // If client is ready, don't show anything (or show a success indicator)
  if (client && address) {
    return (
      <div style={{ padding: '16px', background: 'rgba(40, 167, 69, 0.1)', borderRadius: '8px', marginBottom: '16px' }}>
        <div style={{ fontSize: '14px', color: '#28a745', fontWeight: 600 }}>
          ✅ Wallet Ready ({walletType})
        </div>
        <div style={{ fontSize: '12px', color: 'var(--tg-theme-hint-color, #666)', marginTop: '4px' }}>
          You can now sign transactions
        </div>
      </div>
    );
  }

  // If there's an error, show it
  if (error) {
    return (
      <div style={{ padding: '16px', background: 'rgba(220, 53, 69, 0.1)', borderRadius: '8px', marginBottom: '16px' }}>
        <div style={{ fontSize: '14px', color: '#dc3545', fontWeight: 600 }}>
          ❌ Wallet Error
        </div>
        <div style={{ fontSize: '12px', color: 'var(--tg-theme-hint-color, #666)', marginTop: '4px' }}>
          {error.message}
        </div>
        <div style={{ marginTop: '12px' }}>
          {walletType === 'walletconnect' && (
            <button
              onClick={handleWalletConnectConnect}
              disabled={isInitializing}
              style={{
                padding: '8px 16px',
                background: 'var(--tg-theme-button-color, #3390ec)',
                color: 'var(--tg-theme-button-text-color, #fff)',
                border: 'none',
                borderRadius: '6px',
                fontSize: '14px',
                cursor: 'pointer',
              }}
            >
              Retry WalletConnect
            </button>
          )}
        </div>
      </div>
    );
  }

  // If not ready yet, show loading
  if (!isReady) {
    console.log('WalletManager: Not ready yet, showing loading...');
    return (
      <div style={{ padding: '16px', textAlign: 'center', background: 'rgba(255, 193, 7, 0.1)', borderRadius: '8px', marginBottom: '16px' }}>
        <div style={{ fontSize: '14px', color: '#856404' }}>⏳ Loading wallet...</div>
        <div style={{ fontSize: '11px', color: 'var(--tg-theme-hint-color, #666)', marginTop: '4px' }}>
          Check debug panel for details
        </div>
      </div>
    );
  }

  console.log('WalletManager: Ready! Checking wallet type...');

  // If wallet type is known but no client, show appropriate initialization
  console.log('Checking wallet type:', walletType, 'has mnemonic:', hasMnemonic(), 'has client:', !!client);

  if (walletType === 'local' && !hasMnemonic() && !client) {
    return (
      <div style={{ padding: '16px', background: 'var(--tg-theme-secondary-bg-color, #f0f0f0)', borderRadius: '8px', marginBottom: '16px' }}>
        <div style={{ fontSize: '14px', fontWeight: 600, marginBottom: '8px' }}>
          🔐 Enter Mnemonic to Sign
        </div>
        <div style={{ fontSize: '12px', color: 'var(--tg-theme-hint-color, #666)', marginBottom: '12px' }}>
          Your mnemonic is not saved. Enter it to enable signing.
        </div>
        <textarea
          value={mnemonicInput}
          onChange={(e) => setMnemonicInput(e.target.value)}
          placeholder="Enter your 24-word mnemonic..."
          rows={4}
          style={{
            width: '100%',
            padding: '8px',
            borderRadius: '6px',
            border: '1px solid var(--tg-theme-hint-color, #ccc)',
            fontSize: '12px',
            fontFamily: 'monospace',
            marginBottom: '8px',
          }}
        />
        <button
          onClick={handleMnemonicSubmit}
          disabled={isInitializing || !mnemonicInput.trim()}
          style={{
            width: '100%',
            padding: '10px',
            background: 'var(--tg-theme-button-color, #3390ec)',
            color: 'var(--tg-theme-button-text-color, #fff)',
            border: 'none',
            borderRadius: '6px',
            fontSize: '14px',
            fontWeight: 600,
            cursor: 'pointer',
            opacity: isInitializing || !mnemonicInput.trim() ? 0.5 : 1,
          }}
        >
          {isInitializing ? 'Initializing...' : 'Initialize Wallet'}
        </button>
      </div>
    );
  }

  if (walletType === 'keplr' && !client) {
    return (
      <div style={{ padding: '16px', background: 'var(--tg-theme-secondary-bg-color, #f0f0f0)', borderRadius: '8px', marginBottom: '16px' }}>
        <div style={{ fontSize: '14px', fontWeight: 600, marginBottom: '8px' }}>
          🦊 Connect Keplr
        </div>
        <div style={{ fontSize: '12px', color: 'var(--tg-theme-hint-color, #666)', marginBottom: '12px' }}>
          Connect your Keplr wallet to sign transactions.
        </div>
        <button
          onClick={handleKeplrConnect}
          disabled={isInitializing}
          style={{
            width: '100%',
            padding: '10px',
            background: 'var(--tg-theme-button-color, #3390ec)',
            color: 'var(--tg-theme-button-text-color, #fff)',
            border: 'none',
            borderRadius: '6px',
            fontSize: '14px',
            fontWeight: 600,
            cursor: 'pointer',
            opacity: isInitializing ? 0.5 : 1,
          }}
        >
          {isInitializing ? 'Connecting...' : 'Connect Keplr'}
        </button>
      </div>
    );
  }

  if (walletType === 'walletconnect' && !client) {
    return (
      <div style={{ padding: '16px', background: 'var(--tg-theme-secondary-bg-color, #f0f0f0)', borderRadius: '8px', marginBottom: '16px' }}>
        <div style={{ fontSize: '14px', fontWeight: 600, marginBottom: '8px' }}>
          🔗 Reconnect WalletConnect
        </div>
        <div style={{ fontSize: '12px', color: 'var(--tg-theme-hint-color, #666)', marginBottom: '12px' }}>
          Reconnect to your WalletConnect session to sign transactions.
        </div>
        <button
          onClick={handleWalletConnectConnect}
          disabled={isInitializing}
          style={{
            width: '100%',
            padding: '10px',
            background: 'var(--tg-theme-button-color, #3390ec)',
            color: 'var(--tg-theme-button-text-color, #fff)',
            border: 'none',
            borderRadius: '6px',
            fontSize: '14px',
            fontWeight: 600,
            cursor: 'pointer',
            opacity: isInitializing ? 0.5 : 1,
          }}
        >
          {isInitializing ? 'Connecting...' : 'Reconnect WalletConnect'}
        </button>
      </div>
    );
  }

  // No UI needed if everything is working
  console.log('WalletManager: All good, returning null');
  return null;
}
