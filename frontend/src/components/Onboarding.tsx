import { useState, useEffect } from 'react';
import WebApp from '@twa-dev/sdk';
import { hapticImpact, hapticNotification, showAlert, showConfirm } from '../utils/telegram';
import { WalletManager } from './WalletManager';
import { AccountCreation } from './AccountCreation';
import { useSigningClient } from '../hooks/useSigningClient';
import type { StoredAccount } from '../services/storage';
import { getStoredAccount } from '../services/storage';
import { registerWithAuthz } from '../services/authz';
import { getContractAddress } from '../config/contracts';
import { log } from '../debug';
import './Onboarding.css';

interface OnboardingProps {
  onComplete: () => void;
}

type OnboardingStep = 'welcome' | 'create-account' | 'wallet-connect' | 'set-limit' | 'register';

export function Onboarding({ onComplete }: OnboardingProps) {
  const [account, setAccount] = useState<StoredAccount | null>(null);
  const [currentStep, setCurrentStep] = useState<OnboardingStep>('welcome');
  const [spendingLimit, setSpendingLimit] = useState('100'); // Default 100 NTRN
  const [isRegistering, setIsRegistering] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const { client: signingClient, address: signerAddress } = useSigningClient();

  useEffect(() => {
    WebApp.BackButton.hide();

    // Check if account already exists
    const storedAccount = getStoredAccount();
    setAccount(storedAccount);

    // If wallet is already connected and registered, skip to dashboard
    if (signingClient && signerAddress && storedAccount) {
      const isRegistered = localStorage.getItem('telegram_payments_registered') === 'true';
      if (isRegistered) {
        onComplete();
      }
    }
  }, [signingClient, signerAddress, onComplete]);

  // Auto-proceed from wallet-connect to set-limit once wallet is ready
  useEffect(() => {
    if (currentStep === 'wallet-connect' && signingClient && signerAddress && account && !isResetting) {
      log('Wallet is ready, auto-proceeding to set-limit step');
      log(`Signing client: ${!!signingClient}, Address: ${signerAddress}, Account: ${account.address}`);

      // Verify address matches account
      if (signerAddress === account.address) {
        // Longer delay to ensure wallet is fully settled
        setTimeout(() => {
          setCurrentStep('set-limit');
        }, 1500);
      } else {
        log(`Address mismatch: ${signerAddress} vs ${account.address}`, 'warn');
      }
    }
  }, [currentStep, signingClient, signerAddress, account, isResetting]);

  const handleNext = () => {
    hapticImpact('light');

    switch (currentStep) {
      case 'welcome':
        // Check if account exists
        if (!account) {
          setCurrentStep('create-account');
        } else {
          setCurrentStep('wallet-connect');
        }
        break;
      case 'create-account':
        // Account should be created by AccountCreation component
        if (!account) {
          showAlert('Please create or import an account first');
          return;
        }
        setCurrentStep('wallet-connect');
        break;
      case 'wallet-connect':
        if (!signingClient) {
          showAlert('Please initialize your wallet first');
          return;
        }
        setCurrentStep('set-limit');
        break;
      case 'set-limit':
        if (!spendingLimit || parseFloat(spendingLimit) <= 0) {
          showAlert('Please enter a valid spending limit');
          return;
        }
        setCurrentStep('register');
        break;
    }
  };

  const handleAccountCreated = async () => {
    const newAccount = getStoredAccount();
    setAccount(newAccount);

    // Clear resetting flag if it was set
    setIsResetting(false);

    setCurrentStep('wallet-connect');

    // Trigger wallet reinitialization by calling reinitialize from the hook
    // This ensures the signing client picks up the newly created wallet
    log(`Account created: ${newAccount?.address}`);
    log('Moving to wallet-connect step...');
  };

  const handleResetWallet = () => {
    showConfirm(
      'Are you sure you want to connect a different wallet? Your current wallet settings will be cleared.',
      (confirmed) => {
        if (confirmed) {
          hapticImpact('medium');
          log('Resetting wallet settings...');

          setIsResetting(true);

          // Clear all stored data
          localStorage.removeItem('telegram_payments_account');
          localStorage.removeItem('telegram_payments_mnemonic');
          localStorage.removeItem('telegram_payments_wallet_type');
          localStorage.removeItem('telegram_payments_registered');

          // Reset state
          setAccount(null);

          // Go back to account creation step
          setCurrentStep('create-account');

          // Allow auto-proceed again after reset is complete
          setTimeout(() => {
            setIsResetting(false);
            log('Wallet settings cleared. Ready to connect new wallet.');
          }, 500);
        }
      }
    );
  };

  const handleRegister = async () => {
    if (!signingClient || !signerAddress) {
      showAlert('Wallet not initialized');
      return;
    }

    // Get Telegram user info
    const user = WebApp.initDataUnsafe.user;
    if (!user) {
      showAlert('Could not get Telegram user info');
      return;
    }

    const tgHandle = user.username || `user_${user.id}`;

    // Get contract address
    let contractAddress: string;
    try {
      contractAddress = getContractAddress('payments');
    } catch (err) {
      showAlert('Contract address not configured. Please set it in settings.');
      return;
    }

    setIsRegistering(true);
    hapticImpact('medium');

    try {
      log(`Starting registration for @${tgHandle}...`);

      // Register user and create authz grant in one transaction
      const result = await registerWithAuthz(signingClient, {
        userAddress: signerAddress,
        contractAddress,
        tgHandle,
        spendLimit: spendingLimit,
      });

      log(`✅ Registration complete! Tx: ${result.transactionHash}`);
      hapticNotification('success');

      // Wait a bit for user to see success
      await new Promise(resolve => setTimeout(resolve, 1000));

      onComplete();
    } catch (err) {
      console.error('Registration failed:', err);
      log(`❌ Registration failed: ${(err as Error).message}`, 'error');
      hapticNotification('error');
      showAlert(`Registration failed: ${(err as Error).message}`);
    } finally {
      setIsRegistering(false);
    }
  };

  const getStepNumber = () => {
    const steps: Record<OnboardingStep, number> = {
      welcome: 1,
      'create-account': 2,
      'wallet-connect': 3,
      'set-limit': 4,
      register: 5,
    };
    return steps[currentStep];
  };

  return (
    <div className="onboarding">
      {/* Progress indicator */}
      <div className="onboarding-progress">
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{ width: `${(getStepNumber() / 5) * 100}%` }}
          />
        </div>
        <div className="progress-text">Step {getStepNumber()} of 5</div>
      </div>

      {/* Step content */}
      <div className="onboarding-content">
        {currentStep === 'welcome' && (
          <div className="onboarding-step welcome-step">
            <div className="step-icon">💸</div>
            <h1>Welcome to Telegram Payments</h1>
            <p className="intro-text">
              Send and receive cryptocurrency payments directly through Telegram, powered by the Cosmos ecosystem.
            </p>
            <div className="feature-list">
              <div className="feature">
                <span className="feature-icon">⚡</span>
                <div>
                  <h3>Instant Payments</h3>
                  <p>Send crypto to any Telegram user instantly</p>
                </div>
              </div>
              <div className="feature">
                <span className="feature-icon">🔒</span>
                <div>
                  <h3>Secure & Non-Custodial</h3>
                  <p>Your keys, your crypto. We never hold your funds</p>
                </div>
              </div>
              <div className="feature">
                <span className="feature-icon">🎯</span>
                <div>
                  <h3>Spending Limits</h3>
                  <p>Set custom limits for peace of mind</p>
                </div>
              </div>
            </div>
            <button className="primary-button" onClick={handleNext}>
              Get Started
            </button>
          </div>
        )}

        {currentStep === 'create-account' && (
          <div className="onboarding-step account-step">
            <AccountCreation onAccountCreated={handleAccountCreated} />
          </div>
        )}

        {currentStep === 'wallet-connect' && (
          <div className="onboarding-step wallet-step">
            <div className="step-icon">🔐</div>
            <h2>Connect Your Wallet</h2>
            <p className="step-description">
              Initialize your wallet to start sending and receiving payments.
            </p>

            <WalletManager />

            {signingClient && signerAddress && (
              <div style={{ marginTop: '16px' }}>
                <p style={{ color: 'var(--tg-theme-hint-color)', fontSize: '14px', marginBottom: '12px' }}>
                  ✅ Wallet connected: {signerAddress.substring(0, 12)}...
                </p>
                <button className="primary-button" onClick={handleNext}>
                  Continue
                </button>
              </div>
            )}
          </div>
        )}

        {currentStep === 'set-limit' && (
          <div className="onboarding-step limit-step">
            <div className="step-icon">💰</div>
            <h2>Set Spending Limit</h2>
            <p className="step-description">
              Set a maximum amount that can be spent through Telegram in a single transaction. You can change this anytime.
            </p>

            <div className="limit-input-container">
              <label htmlFor="spending-limit">Spending Limit (NTRN)</label>
              <div className="input-with-suffix">
                <input
                  id="spending-limit"
                  type="number"
                  min="1"
                  step="1"
                  value={spendingLimit}
                  onChange={(e) => setSpendingLimit(e.target.value)}
                  placeholder="100"
                />
                <span className="input-suffix">NTRN</span>
              </div>
              <div className="limit-presets">
                <button onClick={() => setSpendingLimit('10')}>10 NTRN</button>
                <button onClick={() => setSpendingLimit('50')}>50 NTRN</button>
                <button onClick={() => setSpendingLimit('100')}>100 NTRN</button>
                <button onClick={() => setSpendingLimit('500')}>500 NTRN</button>
              </div>
              <p className="helper-text">
                ≈ ${(parseFloat(spendingLimit || '0') * 0.5).toFixed(2)} USD (estimated)
              </p>
            </div>

            <button className="primary-button" onClick={handleNext}>
              Continue
            </button>

            <button
              className="secondary-button"
              onClick={handleResetWallet}
              style={{ marginTop: '12px' }}
            >
              Connect Different Wallet
            </button>
          </div>
        )}

        {currentStep === 'register' && (
          <div className="onboarding-step register-step">
            <div className="step-icon">✅</div>
            <h2>Complete Setup</h2>
            <p className="step-description">
              Register your Telegram handle with the blockchain to start receiving payments.
            </p>

            <div className="registration-summary">
              <div className="summary-item">
                <span className="label">Telegram Handle:</span>
                <span className="value">@{WebApp.initDataUnsafe.user?.username || `user_${WebApp.initDataUnsafe.user?.id}`}</span>
              </div>
              <div className="summary-item">
                <span className="label">Wallet Address:</span>
                <span className="value mono">{account?.address.substring(0, 20)}...</span>
              </div>
              <div className="summary-item">
                <span className="label">Spending Limit:</span>
                <span className="value">{spendingLimit} NTRN</span>
              </div>
            </div>

            <div className="info-box">
              <p>
                This will create an authorization that allows the contract to spend up to {spendingLimit} NTRN on your behalf for Telegram payments.
              </p>
            </div>

            <button
              className="primary-button"
              onClick={handleRegister}
              disabled={isRegistering}
            >
              {isRegistering ? 'Registering...' : 'Complete Registration'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
