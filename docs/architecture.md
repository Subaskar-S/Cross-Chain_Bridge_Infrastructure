# Cross-Chain Bridge Architecture

## Overview

This document describes the architecture of the Ethereum ↔ Polkadot cross-chain bridge, designed for secure and verifiable bidirectional token transfers using threshold signatures.

## System Components

### 1. Smart Contracts

#### Ethereum Bridge Contract
- **Purpose**: Lock/unlock ERC-20 tokens on Ethereum
- **Key Functions**:
  - `lockTokens(address token, uint256 amount, bytes32 polkadotAddress)`
  - `unlockTokens(bytes32 txHash, bytes[] signatures)`
  - `addValidator(address validator)`
  - `removeValidator(address validator)`
- **Events**:
  - `BridgeLock(address indexed user, address indexed token, uint256 amount, bytes32 polkadotAddress)`
  - `BridgeUnlock(address indexed user, address indexed token, uint256 amount)`

#### Polkadot Bridge Pallet
- **Purpose**: Mint/burn wrapped tokens on Polkadot
- **Key Extrinsics**:
  - `mint_tokens(origin, ethereum_tx_hash, signatures)`
  - `burn_tokens(origin, token_id, amount, ethereum_address)`
  - `add_validator(origin, validator_account)`
  - `remove_validator(origin, validator_account)`
- **Events**:
  - `TokensMinted(AccountId, TokenId, Balance, H256)`
  - `TokensBurned(AccountId, TokenId, Balance, H160)`

### 2. Threshold Signature System

#### Key Components
- **Distributed Key Generation (DKG)**: Generate shared keys among validators
- **Threshold Signing**: Create signatures requiring k-of-n validator participation
- **Signature Verification**: Validate threshold signatures on both chains

#### Cryptographic Scheme
- **Primary**: Schnorr signatures for efficiency
- **Fallback**: ECDSA for broader compatibility
- **Threshold**: 2-of-3 for initial deployment (configurable)

### 3. Relayer Service

#### Core Responsibilities
- Monitor events on both chains
- Coordinate threshold signature generation
- Submit proofs to target chains
- Handle validator rotation
- Maintain bridge state consistency

#### Event Processing Pipeline
```
Event Detection → Signature Collection → Proof Generation → Submission → Confirmation
```

### 4. Validator Network

#### Validator Roles
- **Event Witnessing**: Monitor and attest to cross-chain events
- **Signature Generation**: Participate in threshold signature creation
- **Consensus**: Agree on bridge state and operations

#### Validator Requirements
- Stake minimum tokens for participation
- Maintain high uptime (>95%)
- Run both Ethereum and Polkadot nodes
- Participate in threshold signature ceremonies

## Security Model

### Threat Mitigation

#### 1. Validator Collusion
- **Mitigation**: Threshold signatures require majority consensus
- **Detection**: Monitor for unusual signing patterns
- **Response**: Automatic validator rotation and slashing

#### 2. Replay Attacks
- **Mitigation**: Unique transaction hashes and nonces
- **Implementation**: Event deduplication on both chains

#### 3. Front-running
- **Mitigation**: Commit-reveal schemes for sensitive operations
- **Implementation**: Time-locked operations with randomness

#### 4. Bridge Halting
- **Mitigation**: Emergency pause mechanisms
- **Implementation**: Multi-sig admin controls with timelock

### Economic Security

#### Staking Requirements
- Validators must stake tokens to participate
- Slashing for malicious behavior or downtime
- Rewards for honest participation

#### Insurance Fund
- Community-funded insurance for bridge failures
- Automatic payouts for verified losses
- Governance-controlled fund management

## Data Flow

### Ethereum → Polkadot Transfer

1. **User Action**: Call `lockTokens()` on Ethereum contract
2. **Event Emission**: Contract emits `BridgeLock` event
3. **Validator Detection**: Validators observe event via relayer
4. **Signature Generation**: Validators create threshold signature
5. **Proof Submission**: Relayer submits proof to Polkadot
6. **Token Minting**: Polkadot pallet mints wrapped tokens
7. **Confirmation**: User receives tokens on Polkadot

### Polkadot → Ethereum Transfer

1. **User Action**: Call `burn_tokens` extrinsic on Polkadot
2. **Event Emission**: Pallet emits `TokensBurned` event
3. **Validator Detection**: Validators observe event via relayer
4. **Signature Generation**: Validators create threshold signature
5. **Proof Submission**: Relayer submits proof to Ethereum
6. **Token Unlocking**: Ethereum contract unlocks original tokens
7. **Confirmation**: User receives tokens on Ethereum

## Performance Characteristics

### Throughput
- **Target**: 100 transactions per minute
- **Bottleneck**: Threshold signature generation
- **Optimization**: Batch processing and signature aggregation

### Latency
- **Ethereum → Polkadot**: ~5-10 minutes
- **Polkadot → Ethereum**: ~3-7 minutes
- **Factors**: Block confirmations, validator response time

### Costs
- **Ethereum Gas**: ~150,000 gas per operation
- **Polkadot Fees**: Minimal transaction fees
- **Validator Rewards**: 0.1% of transferred value

## Monitoring and Observability

### Key Metrics
- Bridge volume and transaction count
- Validator uptime and performance
- Signature generation latency
- Failed transaction rates

### Alerting
- Validator downtime alerts
- Unusual transaction patterns
- Bridge pause conditions
- Security incident notifications

## Implementation Details

### Database Schema
```sql
-- Ethereum transactions
CREATE TABLE ethereum_locks (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(42) NOT NULL,
    token_address VARCHAR(42) NOT NULL,
    amount VARCHAR(78) NOT NULL,
    polkadot_address VARCHAR(66) NOT NULL,
    tx_hash VARCHAR(66) NOT NULL UNIQUE,
    block_number BIGINT NOT NULL,
    processed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Polkadot transactions
CREATE TABLE polkadot_burns (
    id SERIAL PRIMARY KEY,
    user_account VARCHAR(48) NOT NULL,
    asset_id INTEGER NOT NULL,
    amount VARCHAR(78) NOT NULL,
    ethereum_recipient VARCHAR(42) NOT NULL,
    tx_hash VARCHAR(66) NOT NULL UNIQUE,
    block_number INTEGER NOT NULL,
    processed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Bridge state tracking
CREATE TABLE bridge_state (
    id SERIAL PRIMARY KEY,
    key VARCHAR(50) NOT NULL UNIQUE,
    value VARCHAR(100) NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW()
);
```

### API Endpoints
```
GET  /health                    - Health check
GET  /status                    - Bridge status
GET  /stats                     - Bridge statistics
GET  /transactions              - Transaction history
GET  /transactions/:hash        - Specific transaction
GET  /validators                - Validator information
GET  /tokens                    - Supported tokens
GET  /metrics                   - Prometheus metrics
WS   /ws                        - Real-time events
```

### Configuration Management
```toml
[ethereum]
rpc_url = "https://mainnet.infura.io/v3/YOUR_KEY"
ws_url = "wss://mainnet.infura.io/ws/v3/YOUR_KEY"
chain_id = 1
bridge_contract = "0x..."
confirmations = 12

[polkadot]
ws_url = "wss://rpc.polkadot.io"
confirmations = 6

[threshold]
scheme = "ecdsa"
threshold = 2
total_validators = 3
key_size = 256

[validator]
validator_id = "validator_0"
enabled = true
private_key = "0x..."
```

## Upgrade and Governance

### Contract Upgrades
- Proxy pattern for Ethereum contracts
- Runtime upgrades for Polkadot pallets
- Multi-sig governance for critical changes

### Parameter Updates
- Threshold signature parameters
- Validator set modifications
- Fee structure adjustments
- Emergency response procedures

## Testing Strategy

### Unit Tests
- Smart contract functionality
- Threshold signature operations
- API endpoint validation
- Database operations

### Integration Tests
- End-to-end bridge transfers
- Validator consensus scenarios
- Error handling and recovery
- Performance under load

### Security Tests
- Signature verification
- Replay attack prevention
- Access control validation
- Economic attack simulations
