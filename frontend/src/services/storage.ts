/**
 * Local Storage Service
 * Handles persistence of account data
 */

const ACCOUNT_KEY = "telegram_payments_account";
const MNEMONIC_KEY = "telegram_payments_mnemonic";
const WALLET_TYPE_KEY = "telegram_payments_wallet_type";

export type WalletType = 'local' | 'keplr' | 'walletconnect';

export interface StoredAccount {
  address: string;
  publicKey: string;
  createdAt: number;
}

/**
 * Save account to local storage (without private key for security)
 */
export function saveAccount(account: Omit<StoredAccount, "createdAt">): void {
  const storedAccount: StoredAccount = {
    ...account,
    createdAt: Date.now(),
  };
  localStorage.setItem(ACCOUNT_KEY, JSON.stringify(storedAccount));
}

/**
 * Get stored account
 */
export function getStoredAccount(): StoredAccount | null {
  const data = localStorage.getItem(ACCOUNT_KEY);
  if (!data) return null;

  try {
    return JSON.parse(data);
  } catch {
    return null;
  }
}

/**
 * Check if account exists
 */
export function hasAccount(): boolean {
  return getStoredAccount() !== null;
}

/**
 * Clear stored account
 */
export function clearAccount(): void {
  localStorage.removeItem(ACCOUNT_KEY);
  localStorage.removeItem(MNEMONIC_KEY);
  localStorage.removeItem(WALLET_TYPE_KEY);
}

/**
 * Save mnemonic to local storage
 * WARNING: This stores the mnemonic in plain text in localStorage.
 * Only use this if you understand the security implications.
 */
export function saveMnemonic(mnemonic: string): void {
  localStorage.setItem(MNEMONIC_KEY, mnemonic);
}

/**
 * Get stored mnemonic
 * WARNING: Returns the mnemonic in plain text.
 */
export function getStoredMnemonic(): string | null {
  return localStorage.getItem(MNEMONIC_KEY);
}

/**
 * Clear stored mnemonic
 */
export function clearMnemonic(): void {
  localStorage.removeItem(MNEMONIC_KEY);
}

/**
 * Check if mnemonic exists
 */
export function hasMnemonic(): boolean {
  return getStoredMnemonic() !== null;
}

/**
 * Save wallet type
 */
export function saveWalletType(walletType: WalletType): void {
  localStorage.setItem(WALLET_TYPE_KEY, walletType);
}

/**
 * Get stored wallet type
 */
export function getStoredWalletType(): WalletType | null {
  return localStorage.getItem(WALLET_TYPE_KEY) as WalletType | null;
}

/**
 * Clear wallet type
 */
export function clearWalletType(): void {
  localStorage.removeItem(WALLET_TYPE_KEY);
}
