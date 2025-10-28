import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { getStoredAccount, getStoredMnemonic, type StoredAccount } from '../services/storage';
import { getSigningCosmWasmClient, type ISigningCosmWasmClient } from '../contracts/baseClient';

export type WalletType = 'local' | 'keplr' | 'walletconnect';

interface WalletContextValue {
  account: StoredAccount | null;
  walletType: WalletType | null;
  signingClient: ISigningCosmWasmClient | null;
  isReady: boolean;

  // Initialize wallet with mnemonic (for local accounts)
  initializeWithMnemonic: (mnemonic: string) => Promise<void>;

  // Initialize wallet with Keplr
  initializeWithKeplr: (chainId?: string) => Promise<void>;

  // Initialize wallet with WalletConnect
  initializeWithWalletConnect: (chainId?: string) => Promise<void>;

  // Disconnect wallet
  disconnect: () => void;
}

const WalletContext = createContext<WalletContextValue | undefined>(undefined);

interface WalletProviderProps {
  children: ReactNode;
  rpcEndpoint?: string;
  chainId?: string;
}

export function WalletProvider({
  children,
  rpcEndpoint = 'https://neutron-rpc.publicnode.com:443',
  chainId = 'neutron-1'
}: WalletProviderProps) {
  const [account, setAccount] = useState<StoredAccount | null>(null);
  const [walletType, setWalletType] = useState<WalletType | null>(null);
  const [signingClient, setSigningClient] = useState<ISigningCosmWasmClient | null>(null);
  const [isReady, setIsReady] = useState(false);

  // Try to auto-initialize on mount
  useEffect(() => {
    const storedAccount = getStoredAccount();
    if (storedAccount) {
      setAccount(storedAccount);

      // Try to auto-initialize if we have a stored mnemonic
      const storedMnemonic = getStoredMnemonic();
      if (storedMnemonic) {
        initializeWithMnemonic(storedMnemonic).catch(console.error);
      } else {
        // Account exists but no mnemonic - user needs to manually initialize
        setIsReady(true);
      }
    } else {
      setIsReady(true);
    }
  }, []);

  const initializeWithMnemonic = async (mnemonic: string) => {
    try {
      // Create wallet from mnemonic
      const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
        prefix: 'neutron'
      });

      // Get accounts to verify
      const accounts = await wallet.getAccounts();
      const walletAccount = accounts[0];

      // Create signing client using InterchainJS
      const client = getSigningCosmWasmClient(wallet as any, rpcEndpoint);

      setAccount(getStoredAccount());
      setWalletType('local');
      setSigningClient(client);
      setIsReady(true);

      console.log('Wallet initialized with mnemonic:', walletAccount.address);
    } catch (error) {
      console.error('Failed to initialize wallet with mnemonic:', error);
      throw error;
    }
  };

  const initializeWithKeplr = async (keplrChainId: string = chainId) => {
    try {
      if (!window.keplr) {
        throw new Error('Keplr extension not found');
      }

      // Enable Keplr for the chain
      await window.keplr.enable(keplrChainId);

      // Get the offline signer
      const offlineSigner = window.keplr.getOfflineSigner(keplrChainId);

      // Create signing client
      const client = getSigningCosmWasmClient(offlineSigner as any, rpcEndpoint);

      setAccount(getStoredAccount());
      setWalletType('keplr');
      setSigningClient(client);
      setIsReady(true);

      console.log('Wallet initialized with Keplr');
    } catch (error) {
      console.error('Failed to initialize wallet with Keplr:', error);
      throw error;
    }
  };

  const initializeWithWalletConnect = async (_wcChainId: string = chainId) => {
    try {
      // For WalletConnect, we'll need to implement a custom signer
      // This is more complex and would require maintaining the WalletConnect session
      // For now, throw an error indicating it's not yet implemented
      throw new Error('WalletConnect signing is not yet implemented. Please use the import mnemonic method or Keplr wallet.');
    } catch (error) {
      console.error('Failed to initialize wallet with WalletConnect:', error);
      throw error;
    }
  };

  const disconnect = () => {
    setAccount(null);
    setWalletType(null);
    setSigningClient(null);
    setIsReady(true);
  };

  const value: WalletContextValue = {
    account,
    walletType,
    signingClient,
    isReady,
    initializeWithMnemonic,
    initializeWithKeplr,
    initializeWithWalletConnect,
    disconnect,
  };

  return (
    <WalletContext.Provider value={value}>
      {children}
    </WalletContext.Provider>
  );
}

export function useWallet() {
  const context = useContext(WalletContext);
  if (context === undefined) {
    throw new Error('useWallet must be used within a WalletProvider');
  }
  return context;
}

// Extend window type for Keplr
declare global {
  interface Window {
    keplr?: any;
  }
}
