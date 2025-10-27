# Telegram Payments Contract

A CosmWasm smart contract that enables Telegram users to send and receive cryptocurrency payments through a bot interface, powered by WAVS (Weighted Aggregated Verifiable Signatures) operators.

## Overview

This contract serves as the on-chain payment rails for a Telegram payment system. It manages the relationship between Telegram handles and blockchain addresses, processes payments between users, and handles pending payments for unregistered recipients.

## Key Concepts

### Account Types

1. **Open Account**: A Telegram user who has registered to receive payments by linking their Telegram handle to a blockchain address.
2. **Funded Account**: A user with an open account who has also authorized the contract to send payments on their behalf.
3. **Unregistered Account**: A Telegram user who hasn't registered yet but may have pending payments waiting for them.

### State Management

The contract maintains several key mappings:

- `OPEN_ACCOUNTS`: Maps Telegram handles → blockchain addresses (for receiving)
- `FUNDED_ACCOUNTS`: Maps blockchain addresses → Telegram handles (for sending)
- `PENDING_PAYMENTS`: Stores payments sent to unregistered users
- `ALLOWED_DENOMS`: Whitelist of accepted token denominations
- `SERVICE_MANAGER` or `ADMIN`: Authorization mechanism (WAVS operators or test admin)

## Main Flows

### 1. Register to Receive

**Purpose**: Link a Telegram handle to a blockchain address to receive payments.

**Flow**:
1. Contract gets `RegisterReceive` (admin variant) or `WavsHandleSignedEnvelope` with registration payload (service manager variant)
2. Contract validates the address and stores the mapping in `OPEN_ACCOUNTS`
3. If there are pending payments for this handle, they are automatically transferred

**Entry Points**:
- `ExecuteMsg::RegisterReceive(RegisterReceiveMsg)` - Called by admin/WAVS operators
- `ExecuteMsg::Wavs(WavsHandleSignedEnvelope)` with `WavsPayload::Register` - Called via WAVS

**State Changes**:
- Adds entry to `OPEN_ACCOUNTS` mapping

**Validations**:
- Telegram handle must not already be registered
- Blockchain address must be valid
- Caller must be authorized (admin or WAVS operators)

### 2. Register to Send

**Purpose**: Authorize the contract to send payments on behalf of a blockchain address.

**Flow**:
1. Contract gets `RegisterSend` signed directly by a user
2. Contract verifies the address matches an existing open account registration
3. Contract creates bidirectional mapping by adding entry to `FUNDED_ACCOUNTS`
4. We assume that they also enable authz grants for payments, but do not enforce that in the registration (it will cause send to fail later)

**Entry Points**:
- `ExecuteMsg::RegisterSend { tg_handle }` - Called directly by the user's wallet

**State Changes**:
- Adds entry to `FUNDED_ACCOUNTS` mapping

**Validations**:
- The `msg.sender` must match the address registered for the given `tg_handle` in `OPEN_ACCOUNTS`
- The address must not already be registered in `FUNDED_ACCOUNTS`

### 3. Send Payment

**Purpose**: Transfer tokens from one Telegram user to another.

**Flow**:
1. Contracts get `SendPayment` (admin variant) or `WavsHandleSignedEnvelope` (service manager variant)
2. Contract validates:
   - Token denomination is whitelisted
   - Amount is greater than zero
   - Sender has a funded account
3. Contract determines recipient address:
   - **If recipient has an open account**: Transfer directly to their registered address
   - **If recipient is unregistered**: Transfer to contract address and record in `PENDING_PAYMENTS`
4. Contract executes `BankMsg::Send` _from sender address_ (not the contract itself) to transfer tokens

**Entry Points**:
- `ExecuteMsg::SendPayment(SendPaymentMsg)` - Called by admin/WAVS operators
- `ExecuteMsg::Wavs(WavsHandleSignedEnvelope)` with `WavsPayload::SendPayment` - Called via WAVS

**State Changes**:
- If recipient is unregistered: Adds/updates entry in `PENDING_PAYMENTS`
- Transfers tokens via `BankMsg::Send`

**Validations**:
- Sender must have a funded account (exists in both `OPEN_ACCOUNTS` and `FUNDED_ACCOUNTS`)
- Token denomination must be in `ALLOWED_DENOMS`
- Amount must be greater than zero
- Caller must be authorized (admin or WAVS operators)

## Query Functions

### `AddrByTg { handle: String }`
Returns the blockchain address associated with a Telegram handle (if registered).

**Response**: `ChainAddrResponse { addr: Option<String> }`

### `TgByAddr { account: String }`
Returns the Telegram handle associated with a blockchain address (if it's a funded account).

**Response**: `TgHandleResponse { handle: Option<String> }`

### `PendingPayments { handle: String }`
Returns all pending payments for an unregistered Telegram handle.

**Response**: `Vec<Coin>`

### `AllowedDenoms {}`
Returns the list of whitelisted token denominations.

**Response**: `Vec<String>`

### `Admin {}`
Returns the admin address (if using admin auth mode).

**Response**: `AdminResponse { admin: Option<String> }`

### `Wavs(WavsServiceManager {})`
Returns the WAVS service manager address (if using WAVS auth mode).

**Response**: `String`

## Authorization Modes

The contract supports two authorization modes, configured during instantiation:

### 1. Service Manager (Production)
Uses WAVS operators for decentralized validation of user commands. The contract validates signatures through the `SERVICE_MANAGER` contract before executing privileged operations.

### 2. Admin (Testing)
A single admin address is authorized to execute privileged operations. Used for testing and development.


## Security Considerations

1. **Authorization**: All privileged operations (register receive, send payment) must be called by authorized parties (WAVS operators or admin).

2. **No Overwrites**: The contract prevents overwriting existing registrations to avoid account hijacking.

3. **Validation**: All addresses are validated before storage, and all operations check for proper authorization.

4. **Whitelisting**: Only pre-approved token denominations can be used for payments.

5. **Pending Payments**: Payments to unregistered users are held by the contract until the recipient registers.

## Known Limitations & TODOs

1. **Payment Source**: The contract currently cannot send tokens directly from a user's address. It would need to either:
   - Hold tokens in the contract
   - Use authorization grants (StargateMsg)
   - Implement a different payment mechanism

2. **Pending Payment Distribution**: When a user registers to receive, pending payments should be automatically transferred, but this is not yet implemented (marked as TODO in line 105 of `execute.rs`).

3. **Error Messages**: Several error messages need improvement for better user experience (marked as TODO in lines 31, 47, 60, 132 of `execute.rs`).

## Integration with WAVS

Please see `UserFlows.md`[../../../docs/UserFlows.md] `Architecture.md`[../../../docs/Architecture.md] for more details on the integration with WAVS.

