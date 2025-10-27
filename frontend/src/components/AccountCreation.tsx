import { useState, useEffect } from 'react';
import WebApp from '@twa-dev/sdk';
import { generateKeyPair, importKeyPair } from '../services/crypto';
import type { KeyPair } from '../services/crypto';
import { saveAccount } from '../services/storage';
import { showAlert, showConfirm, hapticImpact, hapticNotification } from '../utils/telegram';
import { connectWalletConnect, checkForNewSession, initWalletConnect, debugWalletConnectStorage } from '../services/walletconnect';
import './AccountCreation.css';

interface AccountCreationProps {
  onAccountCreated: () => void;
}

type FlowMode = 'welcome' | 'create' | 'import' | 'connect';

export function AccountCreation({ onAccountCreated }: AccountCreationProps) {
  const [mode, setMode] = useState<FlowMode>(() => {
    // Restore mode from localStorage if we were connecting
    const saved = localStorage.getItem('wc_connecting_state');
    if (saved) {
      const state = JSON.parse(saved);
      return state.mode || 'welcome';
    }
    return 'welcome';
  });
  const [keyPair, setKeyPair] = useState<KeyPair | null>(null);
  const [showMnemonic, setShowMnemonic] = useState(false);
  const [copied, setCopied] = useState<'mnemonic' | 'address' | null>(null);
  const [importInput, setImportInput] = useState('');
  const [isConnecting, setIsConnecting] = useState(() => {
    const saved = localStorage.getItem('wc_connecting_state');
    if (saved) {
      const state = JSON.parse(saved);
      return state.isConnecting || false;
    }
    return false;
  });
  const [connectingWallet, setConnectingWallet] = useState<string | null>(() => {
    const saved = localStorage.getItem('wc_connecting_state');
    if (saved) {
      const state = JSON.parse(saved);
      return state.connectingWallet || null;
    }
    return null;
  });
  const [wcUri, setWcUri] = useState<string | null>(() => {
    const saved = localStorage.getItem('wc_connecting_state');
    if (saved) {
      const state = JSON.parse(saved);
      return state.wcUri || null;
    }
    return null;
  });
  const [wcCopied, setWcCopied] = useState(false);
  const [isChecking, setIsChecking] = useState(false);
  const [debugInfo, setDebugInfo] = useState<string>('');
  const [sessionFound, setSessionFound] = useState(false);

  // Setup MainButton when keys are generated
  useEffect(() => {
    if (keyPair && mode === 'create') {
      WebApp.MainButton.setText("I've Saved My Mnemonic");
      WebApp.MainButton.show();
      WebApp.MainButton.onClick(handleSaveAccount);
      WebApp.MainButton.enable();

      return () => {
        WebApp.MainButton.hide();
        WebApp.MainButton.offClick(handleSaveAccount);
      };
    }
  }, [keyPair, mode]);

  // Setup BackButton for navigation
  useEffect(() => {
    if (mode !== 'welcome') {
      WebApp.BackButton.show();
      WebApp.BackButton.onClick(handleBack);

      return () => {
        WebApp.BackButton.hide();
        WebApp.BackButton.offClick(handleBack);
      };
    }
  }, [mode]);

  // Save connecting state to localStorage
  useEffect(() => {
    if (isConnecting && wcUri) {
      localStorage.setItem('wc_connecting_state', JSON.stringify({
        mode,
        isConnecting,
        connectingWallet,
        wcUri
      }));
    } else {
      localStorage.removeItem('wc_connecting_state');
    }
  }, [mode, isConnecting, connectingWallet, wcUri]);

  // Initialize WalletConnect if we're restoring a connecting state
  useEffect(() => {
    if (isConnecting && wcUri) {
      initWalletConnect().catch(() => {
        // Ignore initialization errors
      });
    }
  }, []); // Run once on mount

  // Auto-poll for WalletConnect session when connecting
  useEffect(() => {
    if (!isConnecting || !wcUri) {
      return;
    }

    let pollCount = 0;
    const maxPolls = 30; // Poll for up to 1 minute (30 * 2 seconds)
    let lastSessionCount = 0;

    const pollInterval = setInterval(async () => {
      pollCount++;

      try {
        // Check storage first
        const storageInfo = await debugWalletConnectStorage();
        setDebugInfo(`[${pollCount}/${maxPolls}] ${storageInfo.sessionCount}s/${storageInfo.pairingCount}p - checking...`);

        // If we found a session, try to extract the account
        if (storageInfo.sessionCount > 0) {
          // Only try once per session (don't keep retrying the same session)
          if (lastSessionCount === 0) {
            setSessionFound(true);
            setDebugInfo(`✅ SESSION FOUND - Loading account...`);
            lastSessionCount = storageInfo.sessionCount;

            const account = await checkForNewSession('neutron-1');

            if (account) {
              clearInterval(pollInterval);
              localStorage.removeItem('wc_connecting_state');
              setDebugInfo(`🎉 CONNECTED: ${account.address.substring(0, 20)}...`);

              hapticNotification('success');
              await new Promise(resolve => setTimeout(resolve, 500));
              saveAccount({
                address: account.address,
                publicKey: account.publicKey
              });
              onAccountCreated();
            } else {
              setDebugInfo(`❌ FAILED: Could not extract account from session`);
              clearInterval(pollInterval); // Stop - no point retrying
            }
          } else {
            // Already tried this session
            setDebugInfo(`⏸️ Waiting (already processed session)`);
          }
        }
        // If no session yet, keep polling
      } catch (error: any) {
        const errorMsg = error?.message || String(error);
        setDebugInfo(`ERROR: ${errorMsg}`);
        clearInterval(pollInterval); // Stop on error
      }

      if (pollCount >= maxPolls) {
        clearInterval(pollInterval);
        setDebugInfo('Timeout - no session found. Click "Check Connection".');
      }
    }, 2000); // Poll every 2 seconds

    return () => clearInterval(pollInterval);
  }, [isConnecting, wcUri, onAccountCreated]);

  const handleBack = () => {
    hapticImpact('light');
    setMode('welcome');
    setKeyPair(null);
    setShowMnemonic(false);
    setImportInput('');
    setIsConnecting(false);
    setConnectingWallet(null);
    setWcUri(null);
    setWcCopied(false);
    localStorage.removeItem('wc_connecting_state');
  };

  const handleCheckConnection = async () => {
    hapticImpact('medium');
    setIsChecking(true);
    setDebugInfo('Initializing WalletConnect...');

    try {
      // Make sure WalletConnect is initialized
      await initWalletConnect();
      setDebugInfo('Checking for sessions...');

      // Debug: show what's in storage
      const storageInfo = await debugWalletConnectStorage();
      setDebugInfo(`Found ${storageInfo.sessionCount} sessions, ${storageInfo.pairingCount} pairings`);

      // Wait a bit for the session to propagate
      await new Promise(resolve => setTimeout(resolve, 1000));

      const account = await checkForNewSession('cosmoshub-4');

      if (account) {
        setDebugInfo(`Found: ${account.address.substring(0, 20)}...`);
        localStorage.removeItem('wc_connecting_state');
        hapticNotification('success');
        await new Promise(resolve => setTimeout(resolve, 500));
        saveAccount({
          address: account.address,
          publicKey: account.publicKey
        });
        onAccountCreated();
      } else {
        setDebugInfo(`No session. Sessions: ${storageInfo.sessionCount}, Pairings: ${storageInfo.pairingCount}`);
        hapticNotification('warning');
        showAlert('No connection found. Make sure you:\n1. Approved in ' + connectingWallet + '\n2. Didn\'t close the wallet app\n\nThen click Check Connection again.');
      }
    } catch (error: any) {
      setDebugInfo('Error: ' + (error?.message || 'Unknown error'));
      hapticNotification('error');
      showAlert('Error checking connection:\n' + (error?.message || 'Unknown error'));
    } finally {
      setIsChecking(false);
    }
  };

  const handleCancelConnection = () => {
    hapticImpact('light');
    localStorage.removeItem('wc_connecting_state');
    setIsConnecting(false);
    setConnectingWallet(null);
    setWcUri(null);
    setWcCopied(false);
  };

  const handleGenerateAccount = async () => {
    hapticImpact('medium');
    setMode('create');
    try {
      const newKeyPair = await generateKeyPair();
      setKeyPair(newKeyPair);
      setShowMnemonic(true);
    } catch (err) {
      console.error('Failed to generate key pair:', err);
      hapticNotification('error');
      showAlert('Failed to generate account. Please try again.');
    }
  };

  const handleImportMnemonic = async () => {
    const trimmedMnemonic = importInput.trim();

    // Validate mnemonic has 24 words
    const words = trimmedMnemonic.split(/\s+/);
    if (words.length !== 24) {
      hapticNotification('error');
      showAlert('Please enter a valid 24-word mnemonic phrase');
      return;
    }

    hapticImpact('medium');
    try {
      const importedKeyPair = await importKeyPair(trimmedMnemonic);
      hapticNotification('success');
      saveAccount({
        address: importedKeyPair.address,
        publicKey: importedKeyPair.publicKey
      });
      onAccountCreated();
    } catch (err) {
      console.error('Failed to import mnemonic:', err);
      hapticNotification('error');
      showAlert('Failed to import mnemonic. Please check your phrase and try again.');
    }
  };

  const handleConnectWallet = async (walletType: 'keplr' | 'leap') => {
    // Prevent multiple simultaneous connection attempts
    if (isConnecting) {
      return;
    }

    setIsConnecting(true);
    setConnectingWallet(walletType === 'keplr' ? 'Keplr' : 'Leap');
    setWcUri(null);
    setWcCopied(false);
    setDebugInfo('');
    hapticImpact('medium');

    try {
      // Initialize WalletConnect and get URI
      const { uri, account } = await connectWalletConnect('neutron-1');

      // Store the URI for display
      setWcUri(uri);

      console.log('⏳ Waiting for user to approve in wallet...');

      // Wait for the user to approve in the wallet
      const walletAccount = await account;

      console.log('✅ Wallet approved! Account:', walletAccount);

      hapticNotification('success');
      saveAccount({
        address: walletAccount.address,
        publicKey: walletAccount.publicKey
      });
      onAccountCreated();
    } catch (err: any) {
      console.error(`❌ Failed to connect ${walletType}:`, err);
      hapticNotification('error');
      showAlert(err?.message || `Failed to connect to ${walletType === 'keplr' ? 'Keplr' : 'Leap'}. Please try again.`);
      setIsConnecting(false);
      setConnectingWallet(null);
      setWcUri(null);
    }
  };

  const copyWcUri = async () => {
    if (!wcUri) return;
    try {
      await navigator.clipboard.writeText(wcUri);
      hapticNotification('success');
      setWcCopied(true);
      setTimeout(() => setWcCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
      hapticNotification('error');
      showAlert('Failed to copy to clipboard');
    }
  };


  const handleSaveAccount = () => {
    if (!keyPair) return;

    showConfirm(
      'Have you saved your mnemonic? It will not be shown again!',
      (confirmed) => {
        if (confirmed) {
          hapticNotification('success');
          saveAccount({
            address: keyPair.address,
            publicKey: keyPair.publicKey
          });
          onAccountCreated();
        } else {
          hapticNotification('warning');
        }
      }
    );
  };

  const copyToClipboard = async (text: string, type: 'mnemonic' | 'address') => {
    try {
      await navigator.clipboard.writeText(text);
      hapticNotification('success');
      setCopied(type);
      setTimeout(() => setCopied(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
      hapticNotification('error');
      showAlert('Failed to copy to clipboard');
    }
  };

  // Welcome screen with three options
  if (mode === 'welcome') {
    return (
      <div className="account-creation">
        <div className="welcome-card">
          <h1>Welcome to Telegram Payments</h1>
          <p>Create your Cosmos account to start sending and receiving payments via Telegram.</p>

          <div className="option-buttons">
            <button className="primary-button" onClick={handleGenerateAccount}>
              Create New Account
            </button>
            <button className="primary-button" onClick={() => setMode('import')}>
              Import Mnemonic
            </button>
            <button className="primary-button" onClick={() => setMode('connect')}>
              Connect External Wallet
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Import mnemonic flow
  if (mode === 'import') {
    return (
      <div className="account-creation">
        <div className="welcome-card">
          <h2>Import Mnemonic</h2>
          <p>Enter your 24-word mnemonic phrase to restore your account.</p>

          <div className="import-section">
            <label>Mnemonic Phrase</label>
            <textarea
              className="mnemonic-input"
              placeholder="Enter your 24-word mnemonic phrase..."
              value={importInput}
              onChange={(e) => setImportInput(e.target.value)}
              rows={4}
            />
          </div>

          <button
            className="primary-button"
            onClick={handleImportMnemonic}
            disabled={!importInput.trim()}
          >
            Import Account
          </button>
        </div>
      </div>
    );
  }

  // Connect external wallet flow
  if (mode === 'connect') {
    return (
      <div className="account-creation">
        <div className="welcome-card">
          <h2>Connect External Wallet</h2>
          {isConnecting && wcUri ? (
            <>
              <div className="wc-connection-box">
                <h3>Connect to {connectingWallet}</h3>

                {!sessionFound && (
                  <>
                    <p style={{ textAlign: 'center', marginBottom: '20px', color: 'var(--tg-theme-hint-color, #666)' }}>
                      Open the link below to connect your wallet
                    </p>

                    {(() => {
                      // Create the connect URL with wallet type
                      const baseUrl = window.location.origin;
                      const connectUrl = `${baseUrl}/connect?uri=${encodeURIComponent(wcUri)}&wallet=${connectingWallet?.toLowerCase()}`;

                      return (
                        <>
                          <a
                            href={connectUrl}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="primary-button"
                            style={{ display: 'block', textAlign: 'center', textDecoration: 'none', marginBottom: '20px' }}
                            onClick={() => hapticImpact('medium')}
                          >
                            Open {connectingWallet} Connection
                          </a>

                          <div className="wc-uri-box" style={{ marginTop: '20px' }}>
                            <label style={{ display: 'block', marginBottom: '8px', fontSize: '13px', fontWeight: 600 }}>
                              Or copy this link:
                            </label>
                            <code className="wc-uri">{connectUrl}</code>
                            <button
                              className="copy-button"
                              onClick={() => {
                                navigator.clipboard.writeText(connectUrl);
                                hapticNotification('success');
                                setWcCopied(true);
                                setTimeout(() => setWcCopied(false), 2000);
                              }}
                              style={{ marginTop: '8px' }}
                            >
                              {wcCopied ? '✓ Copied!' : 'Copy Link'}
                            </button>
                          </div>
                        </>
                      );
                    })()}

                    <div className="wc-waiting" style={{ marginTop: '20px' }}>
                      ⏳ Waiting for approval in {connectingWallet}...
                    </div>
                  </>
                )}

                {debugInfo && (
                  <div style={{
                    marginTop: sessionFound ? '40px' : '16px',
                    padding: '16px 20px',
                    background: debugInfo.includes('CONNECTED') || debugInfo.includes('SUCCESS')
                      ? 'rgba(40, 167, 69, 0.15)'
                      : debugInfo.includes('ERROR') || debugInfo.includes('FAILED')
                      ? 'rgba(220, 53, 69, 0.15)'
                      : debugInfo.includes('SESSION FOUND')
                      ? 'rgba(0, 123, 255, 0.15)'
                      : 'var(--tg-theme-secondary-bg-color, #f0f0f0)',
                    border: debugInfo.includes('SESSION FOUND') || debugInfo.includes('CONNECTED') || debugInfo.includes('ERROR')
                      ? '2px solid ' + (
                        debugInfo.includes('CONNECTED') ? '#28a745'
                        : debugInfo.includes('ERROR') || debugInfo.includes('FAILED') ? '#dc3545'
                        : '#007bff'
                      )
                      : 'none',
                    borderRadius: '8px',
                    fontSize: sessionFound ? '16px' : '14px',
                    fontWeight: debugInfo.includes('SESSION FOUND') || debugInfo.includes('CONNECTED') || debugInfo.includes('ERROR') ? '600' : '400',
                    color: debugInfo.includes('CONNECTED')
                      ? '#28a745'
                      : debugInfo.includes('ERROR') || debugInfo.includes('FAILED')
                      ? '#dc3545'
                      : debugInfo.includes('SESSION FOUND')
                      ? '#007bff'
                      : 'var(--tg-theme-hint-color, #666)',
                    textAlign: 'center',
                    transition: 'all 0.3s ease',
                  }}>
                    {debugInfo}
                  </div>
                )}

                {!sessionFound && (
                  <>
                    <button
                      className="primary-button"
                      onClick={handleCheckConnection}
                      disabled={isChecking}
                      style={{ marginTop: '16px' }}
                    >
                      {isChecking ? '🔄 Checking...' : '✅ I\'ve Approved - Check Connection'}
                    </button>

                    <button
                      className="secondary-button"
                      onClick={handleCancelConnection}
                      disabled={isChecking}
                      style={{ marginTop: '8px' }}
                    >
                      Cancel & Start Over
                    </button>

                    <p className="hint-text" style={{ marginTop: '12px', fontSize: '12px' }}>
                      After approving in {connectingWallet}, click "Check Connection" or wait for automatic detection.
                    </p>
                  </>
                )}
              </div>
            </>
          ) : isConnecting ? (
            <>
              <p>Initializing connection...</p>
            </>
          ) : (
            <>
              <p>Connect your existing Cosmos wallet to use with Telegram Payments.</p>

              <div className="wallet-options">
                <button className="wallet-button" onClick={() => handleConnectWallet('keplr')} disabled={isConnecting}>
                  <span className="wallet-name">Keplr</span>
                </button>
                <button className="wallet-button" onClick={() => handleConnectWallet('leap')} disabled={isConnecting}>
                  <span className="wallet-name">Leap</span>
                </button>
                <button className="wallet-button" onClick={() => showAlert('More wallets coming soon!')} disabled={isConnecting}>
                  <span className="wallet-name">Other Wallets</span>
                </button>
              </div>

              <p className="hint-text">
                Click on a wallet to connect via WalletConnect. Make sure you have the wallet app installed on your device.
              </p>
            </>
          )}
        </div>
      </div>
    );
  }

  // Create account flow - show generated keys
  if (mode === 'create' && keyPair) {
    return (
      <div className="account-creation">
        <div className="keys-card">
          <h2>Account Created!</h2>
          <p className="warning">
            ⚠️ Save your mnemonic securely. This is the only time it will be shown!
          </p>

          <div className="key-section">
            <label>Address</label>
            <div className="key-display">
              <code>{keyPair.address}</code>
              <button
                className="copy-button"
                onClick={() => copyToClipboard(keyPair.address, 'address')}
              >
                {copied === 'address' ? '✓' : 'Copy'}
              </button>
            </div>
          </div>

          {showMnemonic && (
            <div className="key-section mnemonic-section">
              <label>Mnemonic (24 words)</label>
              <div className="key-display">
                <code className="key-value mnemonic">{keyPair.mnemonic}</code>
                <button
                  className="copy-button"
                  onClick={() => copyToClipboard(keyPair.mnemonic, 'mnemonic')}
                >
                  {copied === 'mnemonic' ? '✓' : 'Copy'}
                </button>
              </div>
            </div>
          )}

          <p className="telegram-button-hint">
            👇 Use the button below to continue
          </p>
        </div>
      </div>
    );
  }

  return null;
}
