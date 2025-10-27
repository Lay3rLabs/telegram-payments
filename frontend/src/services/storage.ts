/**
 * Local Storage Service
 * Handles persistence of account data
 */

const ACCOUNT_KEY = "telegram_payments_account";

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
}
