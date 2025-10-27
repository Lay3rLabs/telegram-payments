# Architecture

## System Overview

The Telegram Payments system enables secure, decentralized payments on Cosmos chains through Telegram. The architecture consists of three main components working together: a Telegram Bot for user interaction, a WAVS Operator Set for secure transaction validation, and Payment Rails smart contracts for on-chain execution.

## High-Level Architecture Diagram

```mermaid
graph TB
    subgraph "User Interface"
        TU[Telegram User]
        Wallet[User Wallet]
        MiniApp[Telegram Mini-App]
    end
    
    subgraph "Telegram Bot"
        TBot[Backend Server]
    end
    
    subgraph "WAVS Operator Network"
        WAgg[WAVS Aggregator]
        WO1[WAVS Operator 1]
        WO2[WAVS Operator 2]
        WO3[WAVS Operator 3]
    end
    
    subgraph "Neutron Chain"
        Bank[Bank Module]
        PR[Payment Rails Contract]
        SM[Service Manager Contract]
    end
    
    TU -->|/status, /help, /start| TBot
    TU -->|Opens| MiniApp
    MiniApp -->|Connect| Wallet
    Wallet -->|Payment Grant| Bank
    Wallet -->|Register| PR
    
    Bank -->|Payment Events| TBot
    TBot -->|Notifications| TU
    
    TU -->|/receive, /send| WO1
    TU -->|/receive, /send| WO2
    TU -->|/receive, /send| WO3
    
    WO1 -->|Verify & Sign| WAgg
    WO2 -->|Verify & Sign| WAgg
    WO3 -->|Verify & Sign| WAgg
    
    WAgg -->|Submit Transaction| PR
    WAgg -->|Send confirmations| TU
    
    PR -->|Verify Signatures| SM
    PR -->|Execute transfers| Bank
```

## Component Architecture

### 1. Telegram Bot (Quick Operations)

**Purpose**: Handle non-secure, fast user interactions

**Responsibilities**:
- Read-Write-access to TG API (Main Entry Point)
- Provide basic instructions upon `/help` and `/start`
- Handle `/status` command queries
- Listen for on-chain payment events
- Send payment notification messages to users
- Provide links to Mini-App URL

**Technology Stack**:
- Single process bot implementation
- Direct integration with Telegram Bot API
- Read-only blockchain monitoring

**Security Model**: No access to funds or sensitive operations

### 2. WAVS Operator Network

**Purpose**: Decentralized, secure validation of Telegram messages and transaction execution

**Components**:

#### WAVS Operators (Multiple Nodes)
- Read-access to TG API (Listen)
- Verify Telegram message authenticity
- Validate user permissions and limits
- Create and sign messages for blockchain transactions

#### WAVS Aggregator
- Write-access to TG API (Tx Receipts)
- Aggregate signatures from operator quorum
- Submit multi-signed transactions to blockchain
- Send confirmation or error messages back to Telegram Bot (tx response)

**Responsibilities**:
- Handle `/receive` command (register open accounts)
- Handle `/send` command (execute payments)
- Verify sending limits and account status
- Coordinate multi-party transaction signing

**Security Model**:
- Based on slashable restaked assets
- Requires quorum consensus for all operations
- No single point of failure

### 3. Payment Rails Smart Contract

**Purpose**: On-chain execution of payment logic

**Key Functions**:

#### Account Management
- Store mapping: `blockchain_address → telegram_handle(s)`
- Store reverse mapping: `telegram_handle → blockchain_address`
- Track account status: "no account", "open", "funded"

#### Payment Execution
- Receive grant permissions from user wallets
- Execute transfers within approved limits
- Route payments to registered addresses
- Store unclaimed payments in vault

#### Vault Management
- Hold payments for unregistered recipients
- Release funds when recipient registers
- Maintain pending payment records

**Technology Stack**:
- CosmWasm smart contract
- Deployed on each supported Cosmos chain
- Integrates with Cosmos SDK grant module

### 4. Telegram Mini-App

**Purpose**: Secure wallet interactions for funding operations

**Responsibilities**:
- Connect to user's wallet (Keplr, WalletConnect)
- Display account status and balance
- Create register transactions to link blockchain address to Telegram handle
- Create grant transactions for funding
- Revoke grants for defunding
- Add additional funds (top-up)

**User Flows**:
- Register to send (initial grant)
- Top up account (increase grant)
- Defund account (revoke grant)

**Security Model**:
- Direct wallet connection (no intermediary)
- User signs all transactions
- Verifies wallet address matches registration

## Data Flow Diagrams

### Registration Flow (Receive)

```mermaid
sequenceDiagram
    participant User
    participant Ops as WAVS Operators
    participant WAgg as WAVS Aggregator
    participant PR as Payment Rails
    participant Bank as Bank Module
    
    User->>Ops: /receive <address>
    Ops->>WAgg: Coordinate verification
    WAgg->>PR: Register address → telegram mapping
    PR->>Bank: Transfer pending payments (if any)
    PR-->>WAgg: Registration confirmed
    WAgg-->>User: "Open account" confirmation
```

### Send Payment Flow

```mermaid
sequenceDiagram
    participant Sender
    participant Ops as WAVS Operators
    participant WAgg as WAVS Aggregator
    participant PR as Payment Rails
    participant Bank as Bank Module
    
    Sender->>Ops: /send <recipient> <amount> <asset>
    Ops->>Ops: Verify funded account
    Ops->>WAgg: Coordinate verification
    WAgg->>PR: Request Transfer
    
    alt Recipient has open account
        PR->>Bank: Transfer from Sender to Recipient
    else Recipient not registered
        PR->>PR: Record Pending Transaction
        PR->>Bank: Transfer from Sender to Payment Rails
    end
    
    PR-->>WAgg: Transaction confirmed
    WAgg-->>Sender: Confirmation message
```

### Funding Flow (Direct to Blockchain)

```mermaid
sequenceDiagram
    participant User
    participant MiniApp as Mini-App
    participant Wallet
    participant PR as Payment Rails
    participant Bank as Bank Module
    
    User->>MiniApp: Open Mini-App
    MiniApp->>Wallet: Request connection
    Wallet-->>MiniApp: Connected
    MiniApp->>MiniApp: Verify address matches registration
    MiniApp->>Bank: Query current balance
    Bank-->>MiniApp: Display balance
    User->>MiniApp: Set sending limit
    MiniApp->>Wallet: Create grant transaction
    Wallet->>User: Sign transaction
    User->>Wallet: Approve
    opt First time funding
        Wallet->>PR: Submit Register Message
        PR->>PR: Link telegram → address (bidirectional)
    end
    Wallet->>Bank: Submit Grant Message
    Bank->>Bank: Record Grant from Sender to Payment Rails
    Bank-->>MiniApp: Transaction Response
    MiniApp-->>User: "Funded account" notification
```

## Account State Model

```mermaid
stateDiagram-v2
    [*] --> NoAccount: User exists on Telegram
    NoAccount --> OpenAccount: /receive <address>
    OpenAccount --> FundedAccount: Grant transaction via Mini-App
    FundedAccount --> OpenAccount: Revoke grant (defund)
    FundedAccount --> FundedAccount: Top up (increase grant)
    
    note right of NoAccount
        Cannot send or receive
    end note
    
    note right of OpenAccount
        Can receive payments
        Cannot send payments
    end note
    
    note right of FundedAccount
        Can send and receive
        Has spending limit
    end note
```

## Security Architecture

### Multi-Layer Security

1. **Telegram Layer**
   - Bot API authentication
   - Message verification
   - Rate limiting

2. **WAVS Operator Layer**
   - Slashable restaked assets
   - Quorum consensus requirement
   - Byzantine fault tolerance
   - Independent operator verification

3. **Blockchain Layer**
   - Grant-based permissions (not custody)
   - Spending limits enforced on-chain
   - Immutable transaction records
   - User can revoke access anytime

### Attack Mitigation

**Handle Squatting Prevention**:
- Separate registration for sending vs receiving
- Receiving: Telegram message proves handle ownership
- Sending: On-chain transaction proves address ownership
- Both required for full bidirectional mapping

**Fund Security**:
- Users maintain custody of funds
- Operators only have limited grant permissions
- Users can revoke grants at any time
- Spending limits prevent total fund drain

**Operator Collusion**:
- Requires quorum consensus
- Slashable stakes for misbehavior
- Transparent on-chain execution
- Users can monitor all transactions

## Technology Stack

### Backend
- **Telegram Bot**: Rust Axiom Server with Telegram Bot API
- **WAVS Service**: Rust WASI Components

### Smart Contracts
- **Language**: Rust
- **Framework**: CosmWasm
- **Chains**: Neutron, other Cosmos chains

### Frontend
- **Mini-App**: React
- **Wallet Integration**: Keplr, Leap, Cosmostation
- **UI Framework**: Telegram Mini-App SDK

## Deployment Architecture

### Infrastructure Components

1. **Telegram Bot Server**
   - Single instance (can be replicated for HA)
   - Webhook mode
   - Stateless design (query state from blockchain)

2. **WAVS Operator Network**
   - Distributed across multiple operators
   - Each operator runs independent node
   - Aggregator coordinates consensus

3. **Smart Contracts**
   - Deployed per supported chain
   - Immutable code (or governed upgrades)
   - Shared state across operator interactions

4. **Mini-App Hosting**
   - Static site hosting (CDN)
   - Client-side wallet integration
   - No backend required

## Scalability Considerations

- **Horizontal Scaling**: Add more WAVS operators for increased security
- **Chain Support**: Deploy Payment Rails to additional Cosmos chains, WAVS Service can route payments to chain based on requested asset
- **Event Processing**: Async payment notification queue
- **Rate Limiting**: Protect against spam and abuse

## Future Enhancements

- Multi-chain support (auto-route based on asset)
- Multi-asset support per transaction
- Scheduled/recurring payments
- Enhanced privacy features (tg handles to blockchain addresses not public)
