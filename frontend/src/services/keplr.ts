/**
 * Keplr Wallet Integration Service
 * Handles connection to Keplr browser extension
 */

import { toBase64 } from "@cosmjs/encoding";

declare global {
  interface Window {
    keplr?: any;
  }
}

export interface KeplrAccount {
  address: string;
  publicKey: string;
  name?: string;
}

/**
 * Check if Keplr extension is installed
 */
export function isKeplrAvailable(): boolean {
  return typeof window !== "undefined" && !!window.keplr;
}

/**
 * Suggest adding a custom chain to Keplr
 * This is optional and depends on whether you want to support custom chains
 */
export async function suggestChain(
  chainId: string,
  chainName: string,
  rpcEndpoint: string
): Promise<void> {
  if (!window.keplr) {
    throw new Error("Keplr extension not found");
  }

  // This is a basic example - you'll need to customize based on your chain
  await window.keplr.experimentalSuggestChain({
    chainId: chainId,
    chainName: chainName,
    rpc: rpcEndpoint,
    rest: rpcEndpoint.replace(":26657", ":1317"), // Typical pattern
    bip44: {
      coinType: 118, // Cosmos standard
    },
    bech32Config: {
      bech32PrefixAccAddr: chainId === "cosmoshub-4" ? "cosmos" : chainId,
      bech32PrefixAccPub:
        chainId === "cosmoshub-4" ? "cosmospub" : `${chainId}pub`,
      bech32PrefixValAddr:
        chainId === "cosmoshub-4" ? "cosmosvaloper" : `${chainId}valoper`,
      bech32PrefixValPub:
        chainId === "cosmoshub-4" ? "cosmosvaloperpub" : `${chainId}valoperpub`,
      bech32PrefixConsAddr:
        chainId === "cosmoshub-4" ? "cosmosvalcons" : `${chainId}valcons`,
      bech32PrefixConsPub:
        chainId === "cosmoshub-4" ? "cosmosvalconspub" : `${chainId}valconspub`,
    },
    currencies: [
      {
        coinDenom: "ATOM",
        coinMinimalDenom: "uatom",
        coinDecimals: 6,
      },
    ],
    feeCurrencies: [
      {
        coinDenom: "ATOM",
        coinMinimalDenom: "uatom",
        coinDecimals: 6,
      },
    ],
    stakeCurrency: {
      coinDenom: "ATOM",
      coinMinimalDenom: "uatom",
      coinDecimals: 6,
    },
  });
}

/**
 * Connect to Keplr wallet and get account information
 */
export async function connectKeplr(
  chainId: string = "cosmoshub-4"
): Promise<KeplrAccount> {
  if (!window.keplr) {
    throw new Error(
      "Keplr extension not installed. Please install Keplr from https://www.keplr.app/"
    );
  }

  try {
    // Enable the chain
    await window.keplr.enable(chainId);

    // Get the offline signer
    const offlineSigner = window.keplr.getOfflineSigner(chainId);

    // Get accounts
    const accounts = await offlineSigner.getAccounts();

    if (accounts.length === 0) {
      throw new Error("No accounts found in Keplr");
    }

    const account = accounts[0];

    // Get the key info for additional details
    const key = await window.keplr.getKey(chainId);

    return {
      address: account.address,
      publicKey: toBase64(account.pubkey),
      name: key.name,
    };
  } catch (error: unknown) {
    if (error instanceof Error && error.message.includes("Request rejected")) {
      throw new Error("Connection request was rejected by the user");
    }
    throw error;
  }
}

/**
 * Disconnect from Keplr (note: this just clears local state, doesn't actually disconnect the extension)
 */
export function disconnectKeplr(): void {
  // Keplr doesn't have a formal disconnect method
  // This is mainly for clearing local application state
  console.log("Keplr disconnected (local state cleared)");
}

/**
 * Get the currently connected account without prompting
 */
export async function getKeplrAccount(
  chainId: string = "cosmoshub-4"
): Promise<KeplrAccount | null> {
  if (!window.keplr) {
    return null;
  }

  try {
    const key = await window.keplr.getKey(chainId);
    return {
      address: key.bech32Address,
      publicKey: toBase64(key.pubKey),
      name: key.name,
    };
  } catch {
    return null;
  }
}
