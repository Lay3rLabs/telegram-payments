/**
 * Authz Service
 * Handles creation of Cosmos SDK authz grants
 */

import type { ISigningCosmWasmClient } from '../contracts/baseClient';
import { log } from '../debug';
import { SendAuthorization } from 'cosmjs-types/cosmos/bank/v1beta1/authz';
import { Timestamp } from 'cosmjs-types/google/protobuf/timestamp';

export interface AuthzGrantParams {
  granter: string; // User's address
  grantee: string; // Contract address
  spendLimit: string; // Amount in untrn (e.g., "100000000" for 100 NTRN)
  denom: string; // Usually "untrn"
  expiration?: Date; // Optional expiration, default 1 year from now
}

/**
 * Create an authz grant message for SendAuthorization
 * This allows the grantee (contract) to send tokens on behalf of the granter (user)
 */
export function createAuthzGrantMessage(params: AuthzGrantParams) {
  const {
    granter,
    grantee,
    spendLimit,
    denom,
    expiration = new Date(Date.now() + 365 * 24 * 60 * 60 * 1000), // 1 year from now
  } = params;

  log(`Creating authz grant: ${spendLimit} ${denom} for ${grantee}`);

  // Create the SendAuthorization with proper protobuf encoding
  const sendAuthz = SendAuthorization.fromPartial({
    spendLimit: [
      {
        denom,
        amount: spendLimit,
      },
    ],
  });

  // Encode the authorization
  const authorizationBytes = SendAuthorization.encode(sendAuthz).finish();

  // Create expiration timestamp
  const expirationTimestamp = Timestamp.fromPartial({
    seconds: BigInt(Math.floor(expiration.getTime() / 1000)),
    nanos: 0,
  });

  return {
    typeUrl: '/cosmos.authz.v1beta1.MsgGrant',
    value: {
      granter,
      grantee,
      grant: {
        authorization: {
          typeUrl: '/cosmos.bank.v1beta1.SendAuthorization',
          value: authorizationBytes,
        },
        expiration: expirationTimestamp,
      },
    },
  };
}

/**
 * Create authz grant + registerSend in one transaction
 */
export async function registerWithAuthz(
  client: ISigningCosmWasmClient,
  params: {
    userAddress: string;
    contractAddress: string;
    tgHandle: string;
    spendLimit: string; // in NTRN (e.g., "100")
    denom?: string;
  }
) {
  const { userAddress, contractAddress, tgHandle, spendLimit, denom = 'untrn' } = params;

  log(`Registering user ${tgHandle} with ${spendLimit} NTRN limit`);

  // Convert NTRN to untrn (1 NTRN = 1,000,000 untrn)
  const spendLimitMicro = (parseFloat(spendLimit) * 1_000_000).toString();

  try {
    // Create authz grant message
    const authzMsg = createAuthzGrantMessage({
      granter: userAddress,
      grantee: contractAddress,
      spendLimit: spendLimitMicro,
      denom,
    });

    // Create registerSend message
    const registerMsg = {
      typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
      value: {
        sender: userAddress,
        contract: contractAddress,
        msg: Buffer.from(
          JSON.stringify({
            register_send: {
              tg_handle: tgHandle,
            },
          })
        ),
        funds: [],
      },
    };

    log('Signing and broadcasting transaction...');

    // Sign and broadcast both messages in one transaction
    const result = await client.signAndBroadcast(
      userAddress,
      [authzMsg, registerMsg],
      'auto',
      'Register for Telegram Payments'
    );

    log(`✅ Transaction successful! Hash: ${result.transactionHash}`);

    return result;
  } catch (error) {
    log(`❌ Registration failed: ${(error as Error).message}`, 'error');
    throw error;
  }
}

/**
 * Update authz grant limit
 */
export async function updateAuthzLimit(
  client: ISigningCosmWasmClient,
  params: {
    userAddress: string;
    contractAddress: string;
    newLimit: string; // in NTRN
    denom?: string;
  }
) {
  const { userAddress, contractAddress, newLimit, denom = 'untrn' } = params;

  log(`Updating authz limit to ${newLimit} NTRN`);

  const spendLimitMicro = (parseFloat(newLimit) * 1_000_000).toString();

  try {
    // Revoke old grant first
    const revokeMsg = {
      typeUrl: '/cosmos.authz.v1beta1.MsgRevoke',
      value: {
        granter: userAddress,
        grantee: contractAddress,
        msgTypeUrl: '/cosmos.bank.v1beta1.MsgSend',
      },
    };

    // Create new grant
    const grantMsg = createAuthzGrantMessage({
      granter: userAddress,
      grantee: contractAddress,
      spendLimit: spendLimitMicro,
      denom,
    });

    log('Signing and broadcasting limit update...');

    const result = await client.signAndBroadcast(
      userAddress,
      [revokeMsg, grantMsg],
      'auto',
      'Update spending limit'
    );

    log(`✅ Limit updated! Hash: ${result.transactionHash}`);

    return result;
  } catch (error) {
    log(`❌ Limit update failed: ${(error as Error).message}`, 'error');
    throw error;
  }
}

/**
 * Revoke authz grant
 */
export async function revokeAuthz(
  client: ISigningCosmWasmClient,
  params: {
    userAddress: string;
    contractAddress: string;
  }
) {
  const { userAddress, contractAddress } = params;

  log('Revoking authz grant...');

  try {
    const revokeMsg = {
      typeUrl: '/cosmos.authz.v1beta1.MsgRevoke',
      value: {
        granter: userAddress,
        grantee: contractAddress,
        msgTypeUrl: '/cosmos.bank.v1beta1.MsgSend',
      },
    };

    const result = await client.signAndBroadcast(
      userAddress,
      [revokeMsg],
      'auto',
      'Revoke authorization'
    );

    log(`✅ Authorization revoked! Hash: ${result.transactionHash}`);

    return result;
  } catch (error) {
    log(`❌ Revoke failed: ${(error as Error).message}`, 'error');
    throw error;
  }
}
