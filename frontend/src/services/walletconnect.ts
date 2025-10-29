/**
 * WalletConnect Service for Cosmos Wallets
 * Supports connecting to Keplr Mobile, Leap Mobile, and other Cosmos wallets
 */

import SignClient from "@walletconnect/sign-client";
import type { SessionTypes } from "@walletconnect/types";
import { log } from "../debug";

export interface WalletConnectAccount {
  address: string;
  publicKey: string;
  walletName?: string;
}

let signClient: SignClient | null = null;
let currentSession: SessionTypes.Struct | null = null;

/**
 * Initialize WalletConnect SignClient
 */
export async function initWalletConnect(): Promise<SignClient> {
  if (signClient) {
    return signClient;
  }

  signClient = await SignClient.init({
    projectId: "7162d74f9bb1a76f0c3d2bcb7ead4ddb",
    metadata: {
      name: "Telegram Payments",
      description: "Send and receive Cosmos payments via Telegram",
      url: window.location.origin,
      icons: ["https://walletconnect.com/walletconnect-logo.png"],
    },
  });

  // Add event listeners
  signClient.on("session_delete", ({ topic }) => {
    if (currentSession?.topic === topic) {
      currentSession = null;
    }
  });

  // Check for existing sessions on init
  const sessions = signClient.session.getAll();
  log(`Found ${sessions.length} existing WalletConnect sessions`);
  if (sessions.length > 0) {
    currentSession = sessions[sessions.length - 1]; // Use most recent session
    log(`Restored WalletConnect session: ${currentSession.peer.metadata.name}`);
    log(`  Topic: ${currentSession.topic.substring(0, 16)}...`);
    log(`  Accounts: ${currentSession.namespaces.cosmos?.accounts?.length || 0}`);
  }

  return signClient;
}

/**
 * Connect to a Cosmos wallet via WalletConnect
 */
export async function connectWalletConnect(
  chainId: string = "neutron-1"
): Promise<{ uri: string; account: Promise<WalletConnectAccount> }> {
  const client = await initWalletConnect();

  // Clean up any old sessions first
  const oldSessions = client.session.getAll();
  for (const session of oldSessions) {
    try {
      await client.disconnect({
        topic: session.topic,
        reason: {
          code: 6000,
          message: "Starting new connection",
        },
      });
    } catch (error) {
      // Ignore disconnect errors
      console.error("Error disconnecting old session:", error);
    }
  }
  currentSession = null;

  // Define the Cosmos namespace methods we need
  const requiredNamespaces = {
    cosmos: {
      methods: ["cosmos_getAccounts", "cosmos_signDirect", "cosmos_signAmino"],
      chains: [`cosmos:${chainId}`],
      events: ["chainChanged", "accountsChanged"],
    },
  };

  // Create connection
  const { uri, approval } = await client.connect({
    requiredNamespaces,
  });

  if (!uri) {
    throw new Error("Failed to generate WalletConnect URI");
  }

  // Return URI immediately for QR code display
  // and a promise that resolves when connection is approved
  const accountPromise = approval().then(async (session) => {
    currentSession = session;

    // Get the account from the session
    const cosmosNamespace = session.namespaces.cosmos;
    if (
      !cosmosNamespace ||
      !cosmosNamespace.accounts ||
      cosmosNamespace.accounts.length === 0
    ) {
      throw new Error("No Cosmos accounts found in session");
    }

    // Parse the account (format: "cosmos:cosmoshub-4:cosmos1abc...")
    const accountString = cosmosNamespace.accounts[0];
    const address = accountString.split(":")[2];

    // Get public key via cosmos_getAccounts method
    try {
      const accounts = await client.request<
        Array<{ address: string; pubkey: string }>
      >({
        topic: session.topic,
        chainId: `cosmos:${chainId}`,
        request: {
          method: "cosmos_getAccounts",
          params: {},
        },
      });

      const account = accounts.find((acc) => acc.address === address);

      return {
        address,
        publicKey: account?.pubkey || "",
        walletName: session.peer.metadata.name,
      };
    } catch (error) {
      console.log("Failed to get public key from wallet:", error);
      // Fallback if we can't get pubkey
      return {
        address,
        publicKey: "",
        walletName: session.peer.metadata.name,
      };
    }
  });

  return {
    uri,
    account: accountPromise,
  };
}

/**
 * Disconnect WalletConnect session
 */
export async function disconnectWalletConnect(): Promise<void> {
  if (!signClient || !currentSession) {
    return;
  }

  try {
    await signClient.disconnect({
      topic: currentSession.topic,
      reason: {
        code: 6000,
        message: "User disconnected",
      },
    });
  } catch (error) {
    console.log("Error disconnecting WalletConnect session:", error);
    // Ignore disconnect errors
  } finally {
    currentSession = null;
  }
}

/**
 * Get the current WalletConnect session
 */
export function getCurrentSession(): SessionTypes.Struct | null {
  return currentSession;
}

/**
 * Check if WalletConnect is already connected
 */
export function isWalletConnectConnected(): boolean {
  return currentSession !== null;
}

/**
 * Check for new sessions and return account if found
 */
export async function checkForNewSession(
  chainId: string = "neutron-1"
): Promise<WalletConnectAccount | null> {
  // Make sure client is initialized
  const client = await initWalletConnect();

  // Get all sessions directly from storage
  const sessions = client.session.getAll();

  if (sessions.length === 0) {
    return null;
  }

  // Use the most recent session
  const session = sessions[sessions.length - 1];
  currentSession = session;

  // Get account from session
  const availableNamespaces = Object.keys(session.namespaces);
  const cosmosNamespace = session.namespaces.cosmos;

  if (!cosmosNamespace) {
    let foundInfo = "";
    availableNamespaces.forEach((ns) => {
      const nsData = session.namespaces[ns];
      foundInfo += `${ns}: ${nsData.accounts?.length || 0} accounts; `;
    });
    const error = `No cosmos namespace! Found: ${foundInfo || "none"}`;
    throw new Error(error);
  }

  if (!cosmosNamespace.accounts || cosmosNamespace.accounts.length === 0) {
    throw new Error("No accounts in cosmos namespace");
  }

  const accountString = cosmosNamespace.accounts[0];
  const addressParts = accountString.split(":");

  if (addressParts.length < 3) {
    throw new Error(`Invalid account format: ${accountString}`);
  }

  const address = addressParts[2];

  if (!address) {
    throw new Error("Address is empty");
  }

  // Try to get public key with timeout
  try {
    // Add a timeout to prevent hanging
    const pubkeyPromise = client.request<
      Array<{ address: string; pubkey: string }>
    >({
      topic: session.topic,
      chainId: `cosmos:${chainId}`,
      request: {
        method: "cosmos_getAccounts",
        params: {},
      },
    });

    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => reject(new Error("Pubkey request timeout")), 5000); // 5 second timeout
    });

    const accounts = await Promise.race([pubkeyPromise, timeoutPromise]);
    const account = accounts.find((acc) => acc.address === address);

    return {
      address,
      publicKey: account?.pubkey || "",
      walletName: session.peer.metadata.name,
    };
  } catch (error) {
    console.log("Failed to get public key from wallet:", error);
    // Fallback without pubkey - this is OK, we can still use the address
    return {
      address,
      publicKey: "",
      walletName: session.peer.metadata.name,
    };
  }
}

/**
 * Debug function to inspect WalletConnect storage
 */
export type DebugWalletConnectStorageResult =
  | { error: string }
  | {
      sessionCount: number;
      pairingCount: number;
      sessions: {
        topic: string;
        peer: string;
        expiry: string;
        accounts: string[];
      }[];
      pairings: {
        topic: string;
        active: boolean;
        expiry: string;
      }[];
    };

export async function debugWalletConnectStorage(): Promise<DebugWalletConnectStorageResult> {
  if (!signClient) {
    return { error: "SignClient not initialized" };
  }

  const sessions = signClient.session.getAll();
  const pairings = signClient.core.pairing.pairings.getAll();

  return {
    sessionCount: sessions.length,
    pairingCount: pairings.length,
    sessions: sessions.map((s) => ({
      topic: s.topic,
      peer: s.peer.metadata.name,
      expiry: new Date(s.expiry * 1000).toISOString(),
      accounts: s.namespaces.cosmos?.accounts || [],
    })),
    pairings: pairings.map((p) => ({
      topic: p.topic,
      active: p.active,
      expiry: new Date(p.expiry * 1000).toISOString(),
    })),
  };
}

/**
 * Get WalletConnect connection instructions for mobile
 */
export function getWalletConnectionInstructions(
  walletType: "keplr" | "leap",
  wcUri: string
) {
  const walletName = walletType === "keplr" ? "Keplr" : "Leap";

  return {
    walletName,
    uri: wcUri,
    steps: [
      `Open your ${walletName} app`,
      'Tap on "WalletConnect" or the scan icon',
      "Look for the connection request from Telegram Payments",
      "Approve the connection",
    ],
  };
}
