import { useState, useEffect } from 'react';
import WebApp from '@twa-dev/sdk';
import { generateKeyPair } from '../services/mockCrypto';
import type { KeyPair } from '../services/mockCrypto';
import { saveAccount } from '../services/storage';
import './AccountCreation.css';

interface AccountCreationProps {
  onAccountCreated: () => void;
}

export function AccountCreation({ onAccountCreated }: AccountCreationProps) {
  const [keyPair, setKeyPair] = useState<KeyPair | null>(null);
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [copied, setCopied] = useState<'private' | 'public' | 'address' | null>(null);

  // Setup MainButton when keys are generated
  useEffect(() => {
    if (keyPair) {
      WebApp.MainButton.setText("I've Saved My Keys");
      WebApp.MainButton.show();
      WebApp.MainButton.onClick(handleSaveAccount);
      WebApp.MainButton.enable();

      return () => {
        WebApp.MainButton.hide();
        WebApp.MainButton.offClick(handleSaveAccount);
      };
    }
  }, [keyPair]);

  const handleGenerateAccount = () => {
    WebApp.HapticFeedback.impactOccurred('medium');
    const newKeyPair = generateKeyPair();
    setKeyPair(newKeyPair);
    setShowPrivateKey(true);
  };

  const handleSaveAccount = () => {
    if (!keyPair) return;

    WebApp.showConfirm(
      'Have you saved your private key? It will not be shown again!',
      (confirmed) => {
        if (confirmed) {
          WebApp.HapticFeedback.notificationOccurred('success');
          saveAccount({
            address: keyPair.address,
            publicKey: keyPair.publicKey
          });
          onAccountCreated();
        } else {
          WebApp.HapticFeedback.notificationOccurred('warning');
        }
      }
    );
  };

  const copyToClipboard = async (text: string, type: 'private' | 'public' | 'address') => {
    try {
      await navigator.clipboard.writeText(text);
      WebApp.HapticFeedback.notificationOccurred('success');
      setCopied(type);
      setTimeout(() => setCopied(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
      WebApp.HapticFeedback.notificationOccurred('error');
      WebApp.showAlert('Failed to copy to clipboard');
    }
  };

  if (!keyPair) {
    return (
      <div className="account-creation">
        <div className="welcome-card">
          <h1>Welcome to Telegram Payments</h1>
          <p>Create your Cosmos account to start sending and receiving payments via Telegram.</p>
          <button className="primary-button" onClick={handleGenerateAccount}>
            Create Account
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="account-creation">
      <div className="keys-card">
        <h2>Account Created!</h2>
        <p className="warning">
          ⚠️ Save your private key securely. This is the only time it will be shown!
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

        <div className="key-section">
          <label>Public Key</label>
          <div className="key-display">
            <code className="key-value">{keyPair.publicKey}</code>
            <button
              className="copy-button"
              onClick={() => copyToClipboard(keyPair.publicKey, 'public')}
            >
              {copied === 'public' ? '✓' : 'Copy'}
            </button>
          </div>
        </div>

        {showPrivateKey && (
          <div className="key-section private-key-section">
            <label>Private Key</label>
            <div className="key-display">
              <code className="key-value private">{keyPair.privateKey}</code>
              <button
                className="copy-button"
                onClick={() => copyToClipboard(keyPair.privateKey, 'private')}
              >
                {copied === 'private' ? '✓' : 'Copy'}
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
