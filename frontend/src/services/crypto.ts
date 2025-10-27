/**
 * Cosmos Crypto Service
 * Real implementation using @cosmjs libraries
 */

import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { fromBech32, toBase64 } from "@cosmjs/encoding";

export interface KeyPair {
  mnemonic: string;
  privateKey: string;
  publicKey: string;
  address: string;
}

/**
 * Generates a real Cosmos wallet with mnemonic
 */
export async function generateKeyPair(
  prefix: string = "neutron"
): Promise<KeyPair> {
  // Generate wallet with 24-word mnemonic
  const wallet = await DirectSecp256k1HdWallet.generate(24, { prefix });

  // Get the first account
  const accounts = await wallet.getAccounts();
  const account = accounts[0];

  // Get private key (serialize the wallet to get mnemonic)
  const mnemonic = wallet.mnemonic;

  return {
    mnemonic: mnemonic,
    privateKey: mnemonic, // In CosmJS, the mnemonic IS the private key
    publicKey: toBase64(account.pubkey),
    address: account.address,
  };
}

/**
 * Import a wallet from a mnemonic
 */
export async function importKeyPair(
  mnemonic: string,
  prefix: string = "neutron"
): Promise<KeyPair> {
  // Restore wallet from mnemonic
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix,
  });

  // Get the first account
  const accounts = await wallet.getAccounts();
  const account = accounts[0];

  return {
    mnemonic: mnemonic,
    privateKey: mnemonic,
    publicKey: toBase64(account.pubkey),
    address: account.address,
  };
}

/**
 * Validate a Cosmos address
 */
export function isValidAddress(
  address: string,
  expectedPrefix: string = "neutron"
): boolean {
  try {
    const { prefix, data } = fromBech32(address);
    // Check if prefix matches and address has correct length (20 bytes)
    return prefix === expectedPrefix && data.length === 20;
  } catch {
    return false;
  }
}

/**
 * Get the prefix from an address
 */
export function getAddressPrefix(address: string): string | null {
  try {
    const { prefix } = fromBech32(address);
    return prefix;
  } catch {
    return null;
  }
}
