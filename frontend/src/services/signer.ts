/**
 * Signer Service
 * Utilities for creating CosmWasm signing clients from different wallet sources
 */

import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { getSigningCosmWasmClient, type ISigningCosmWasmClient } from '../contracts/baseClient';
import { getCurrentSession, initWalletConnect } from './walletconnect';
import type { SessionTypes } from '@walletconnect/types';
import type { AccountData, OfflineDirectSigner } from '@cosmjs/proto-signing';
import { log } from '../debug';

export const DEFAULT_RPC_ENDPOINT = 'https://neutron-rpc.publicnode.com:443';
export const DEFAULT_CHAIN_ID = 'neutron-1';
export const DEFAULT_PREFIX = 'neutron';

/**
 * Create a signing client from a mnemonic
 */
export async function createSignerFromMnemonic(
  mnemonic: string,
  rpcEndpoint: string = DEFAULT_RPC_ENDPOINT,
  prefix: string = DEFAULT_PREFIX
): Promise<{ client: ISigningCosmWasmClient; address: string }> {
  // Create wallet from mnemonic
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, { prefix });

  // Get the first account
  const accounts = await wallet.getAccounts();
  const account = accounts[0];

  // Create signing client
  const client = getSigningCosmWasmClient(wallet as any, rpcEndpoint);

  return {
    client,
    address: account.address,
  };
}

/**
 * Create a signing client from Keplr wallet
 */
export async function createSignerFromKeplr(
  chainId: string = DEFAULT_CHAIN_ID,
  rpcEndpoint: string = DEFAULT_RPC_ENDPOINT
): Promise<{ client: ISigningCosmWasmClient; address: string }> {
  if (!window.keplr) {
    throw new Error('Keplr extension not found. Please install Keplr from https://www.keplr.app/');
  }

  // Enable Keplr for the chain
  await window.keplr.enable(chainId);

  // Get the offline signer
  const offlineSigner = window.keplr.getOfflineSigner(chainId);

  // Get the first account
  const accounts = await offlineSigner.getAccounts();
  const account = accounts[0];

  // Create signing client
  const client = getSigningCosmWasmClient(offlineSigner as any, rpcEndpoint);

  return {
    client,
    address: account.address,
  };
}

/**
 * Create a WalletConnect offline signer
 */
class WalletConnectSigner implements OfflineDirectSigner {
  private session: SessionTypes.Struct;
  private chainId: string;

  constructor(
    session: SessionTypes.Struct,
    chainId: string
  ) {
    this.session = session;
    this.chainId = chainId;
  }

  async getAccounts(): Promise<readonly AccountData[]> {
    log('WalletConnectSigner.getAccounts() called');
    const cosmosNamespace = this.session.namespaces.cosmos;
    if (!cosmosNamespace || !cosmosNamespace.accounts || cosmosNamespace.accounts.length === 0) {
      throw new Error('No Cosmos accounts found in WalletConnect session');
    }

    log(`Found ${cosmosNamespace.accounts.length} accounts in session`);
    // Parse account (format: "cosmos:neutron-1:neutron1abc...")
    const accountString = cosmosNamespace.accounts[0];
    log(`Parsing account string: ${accountString}`);
    const addressParts = accountString.split(':');
    const address = addressParts[2];
    log(`Parsed address: ${address}`);

    // Use a valid dummy compressed secp256k1 pubkey (33 bytes starting with 0x02)
    // The real pubkey will be provided by the wallet during signing
    const pubkey = new Uint8Array(33);
    pubkey[0] = 0x02; // Valid compressed key marker
    log('Using valid compressed dummy pubkey (real key provided during signing)');

    log(`Returning account: ${address}`);
    return [{
      address,
      pubkey,
      algo: 'secp256k1' as const,
    }];
  }

  async signDirect(signerAddress: string, signDoc: any): Promise<any> {
    const signClient = await initWalletConnect();

    // Convert signDoc to the format expected by WalletConnect
    const signDocJson = {
      chainId: signDoc.chainId,
      accountNumber: signDoc.accountNumber.toString(),
      authInfoBytes: Array.from(signDoc.authInfoBytes),
      bodyBytes: Array.from(signDoc.bodyBytes),
    };

    const result = await signClient.request<{
      signed: {
        accountNumber: string;
        authInfoBytes: number[];
        bodyBytes: number[];
      };
      signature: {
        pub_key: any;
        signature: string;
      };
    }>({
      topic: this.session.topic,
      chainId: `cosmos:${this.chainId}`,
      request: {
        method: 'cosmos_signDirect',
        params: {
          signerAddress,
          signDoc: signDocJson,
        },
      },
    });

    return {
      signed: {
        ...signDoc,
        accountNumber: BigInt(result.signed.accountNumber),
        authInfoBytes: new Uint8Array(result.signed.authInfoBytes),
        bodyBytes: new Uint8Array(result.signed.bodyBytes),
      },
      signature: {
        pub_key: result.signature.pub_key,
        signature: result.signature.signature,
      },
    };
  }
}

/**
 * Create a signing client from WalletConnect session
 */
export async function createSignerFromWalletConnect(
  chainId: string = DEFAULT_CHAIN_ID,
  rpcEndpoint: string = DEFAULT_RPC_ENDPOINT
): Promise<{ client: ISigningCosmWasmClient; address: string }> {
  log('createSignerFromWalletConnect called');

  // IMPORTANT: Initialize WalletConnect to restore sessions from storage
  log('Initializing WalletConnect to restore session...');
  await initWalletConnect();

  const session = getCurrentSession();
  log(`Current WalletConnect session: ${session ? 'found' : 'not found'}`);

  if (!session) {
    throw new Error('No active WalletConnect session. Please connect your wallet first.');
  }

  log('Creating WalletConnect signer...');
  // Create WalletConnect signer
  const signer = new WalletConnectSigner(session, chainId);

  log('Getting accounts from signer...');
  // Get accounts
  const accounts = await signer.getAccounts();
  const account = accounts[0];
  log(`Account address: ${account.address}`);

  log('Creating signing client...');
  // Create signing client
  const client = getSigningCosmWasmClient(signer as any, rpcEndpoint);

  log('✅ WalletConnect signing client created successfully');
  return {
    client,
    address: account.address,
  };
}

/**
 * Prompt user to enter mnemonic via Telegram
 * This is useful when you need the mnemonic but don't have it stored
 */
export function promptForMnemonic(message: string = 'Please enter your 24-word mnemonic'): Promise<string> {
  return new Promise((resolve, reject) => {
    const mnemonic = prompt(message);
    if (mnemonic && mnemonic.trim()) {
      const words = mnemonic.trim().split(/\s+/);
      if (words.length === 24) {
        resolve(mnemonic.trim());
      } else {
        reject(new Error('Invalid mnemonic: must be 24 words'));
      }
    } else {
      reject(new Error('No mnemonic provided'));
    }
  });
}

