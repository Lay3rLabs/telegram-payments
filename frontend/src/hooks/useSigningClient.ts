/**
 * Hook for managing CosmWasm signing client
 */

import { useState, useEffect } from 'react';
import { type ISigningCosmWasmClient } from '../contracts/baseClient';
import { getStoredAccount, getStoredMnemonic, getStoredWalletType } from '../services/storage';
import {
  createSignerFromMnemonic,
  createSignerFromKeplr,
  createSignerFromWalletConnect,
  DEFAULT_RPC_ENDPOINT,
  DEFAULT_CHAIN_ID,
} from '../services/signer';
import { log } from '../debug';

export interface UseSigningClientResult {
  client: ISigningCosmWasmClient | null;
  address: string | null;
  walletType: 'local' | 'keplr' | 'walletconnect' | null;
  isReady: boolean;
  error: Error | null;

  // Initialize with mnemonic
  initializeWithMnemonic: (mnemonic: string) => Promise<void>;

  // Initialize with Keplr
  initializeWithKeplr: () => Promise<void>;

  // Initialize with WalletConnect
  initializeWithWalletConnect: () => Promise<void>;

  // Reinitialize (useful after page reload)
  reinitialize: () => Promise<void>;
}

export function useSigningClient(
  rpcEndpoint: string = DEFAULT_RPC_ENDPOINT,
  chainId: string = DEFAULT_CHAIN_ID
): UseSigningClientResult {
  const [client, setClient] = useState<ISigningCosmWasmClient | null>(null);
  const [address, setAddress] = useState<string | null>(null);
  const [walletType, setWalletType] = useState<'local' | 'keplr' | 'walletconnect' | null>(null);
  const [isReady, setIsReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Auto-initialize on mount
  useEffect(() => {
    reinitialize();
  }, []);

  const reinitialize = async () => {
    try {
      log('🔄 Reinitializing wallet...');
      const storedAccount = getStoredAccount();
      if (!storedAccount) {
        log('No stored account, setting ready');
        setIsReady(true);
        return;
      }

      const storedType = getStoredWalletType();
      const storedMnemonic = getStoredMnemonic();
      log(`Stored wallet type: ${storedType}`);

      // If we have a mnemonic stored, use it
      if (storedMnemonic) {
        log('Reinitializing with stored mnemonic...');
        await initializeWithMnemonic(storedMnemonic);
      }
      // If wallet type is Keplr, try to connect to Keplr
      else if (storedType === 'keplr') {
        log('Reinitializing with Keplr...');
        await initializeWithKeplr();
      }
      // If wallet type is WalletConnect, try to use existing session
      else if (storedType === 'walletconnect') {
        log('Reinitializing with WalletConnect...');
        await initializeWithWalletConnect();
      }
      // Otherwise, just mark as ready (user will need to manually initialize)
      else {
        log('No specific wallet type, just setting address and ready');
        setAddress(storedAccount.address);
        setWalletType(storedType);
        setIsReady(true);
      }
    } catch (err) {
      log(`Failed to reinitialize wallet: ${(err as Error).message}`, 'error');
      console.error('Failed to reinitialize wallet:', err);
      setError(err as Error);
      setIsReady(true);
    }
  };

  const initializeWithMnemonic = async (mnemonic: string) => {
    try {
      setError(null);
      const { client: signingClient, address: walletAddress } = await createSignerFromMnemonic(
        mnemonic,
        rpcEndpoint
      );

      setClient(signingClient);
      setAddress(walletAddress);
      setWalletType('local');
      setIsReady(true);
    } catch (err) {
      console.error('Failed to initialize with mnemonic:', err);
      setError(err as Error);
      setIsReady(true); // Set ready even on error
      throw err;
    }
  };

  const initializeWithKeplr = async () => {
    try {
      setError(null);
      const { client: signingClient, address: walletAddress } = await createSignerFromKeplr(
        chainId,
        rpcEndpoint
      );

      setClient(signingClient);
      setAddress(walletAddress);
      setWalletType('keplr');
      setIsReady(true);
    } catch (err) {
      console.error('Failed to initialize with Keplr:', err);
      setError(err as Error);
      setIsReady(true); // Set ready even on error
      throw err;
    }
  };

  const initializeWithWalletConnect = async () => {
    try {
      setError(null);
      log('🔗 Initializing WalletConnect signer...');
      const { client: signingClient, address: walletAddress } = await createSignerFromWalletConnect(
        chainId,
        rpcEndpoint
      );

      log(`✅ WalletConnect signer created: ${walletAddress}`);
      setClient(signingClient);
      setAddress(walletAddress);
      setWalletType('walletconnect');
      setIsReady(true);
      log('✅ WalletConnect signer ready!');
    } catch (err) {
      log(`❌ Failed to initialize with WalletConnect: ${(err as Error).message}`, 'error');
      console.error('Failed to initialize with WalletConnect:', err);
      setError(err as Error);
      setIsReady(true); // Set ready even on error so UI can show error state
      throw err;
    }
  };

  return {
    client,
    address,
    walletType,
    isReady,
    error,
    initializeWithMnemonic,
    initializeWithKeplr,
    initializeWithWalletConnect,
    reinitialize,
  };
}
