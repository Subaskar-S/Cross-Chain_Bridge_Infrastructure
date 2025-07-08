# Cross-Chain Bridge: Ethereum ↔ Polkadot

A secure and verifiable bridge enabling bidirectional token/asset transfers between Ethereum and Polkadot using threshold signatures.

## 🏗️ Architecture Overview

This bridge implements a secure cross-chain asset transfer mechanism with the following components:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│    Ethereum     │    │     Bridge      │    │    Polkadot     │
│   Smart         │◄──►│   Relayer       │◄──►│    Pallet       │
│   Contracts     │    │   Service       │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   Threshold     │
                    │   Signatures    │
                    │   (k-of-n)      │
                    └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   API Server    │
                    │   & WebSocket   │
                    └─────────────────┘
```

- **Threshold Signatures**: k-of-n validator consensus using Schnorr/ECDSA
- **Smart Contracts**: Ethereum (Solidity) and Polkadot (Substrate) contracts
- **Rust Relayer**: Off-chain service for event monitoring and proof relay
- **API Layer**: REST/WebSocket endpoints for monitoring and status

## 📁 Project Structure

```
├── contracts/          # Smart contracts for both chains
│   ├── ethereum/       # Solidity contracts for Ethereum
│   └── substrate/      # Substrate pallets/contracts for Polkadot
├── relayer/           # Rust-based off-chain relayer service
├── threshold/         # Threshold signature implementation
├── api/              # Backend API for monitoring and UI
├── tests/            # Integration and unit tests
├── docs/             # Architecture and deployment documentation
└── scripts/          # Deployment and utility scripts
```

## 🔄 Bridge Flow

### Ethereum → Polkadot
1. User locks ERC-20 tokens in Ethereum contract
2. Contract emits `BridgeLock` event
3. Validators observe event and create threshold signature
4. Relayer submits proof to Polkadot
5. Wrapped tokens are minted on Polkadot

### Polkadot → Ethereum
1. User burns wrapped tokens on Polkadot
2. Pallet emits `BridgeBurn` event
3. Validators observe event and create threshold signature
4. Relayer submits proof to Ethereum
5. Original tokens are unlocked on Ethereum

## 🔐 Security Features

- **Threshold Signatures**: Requires k-of-n validator consensus
- **Event Verification**: Cryptographic proof validation on both chains
- **Validator Rotation**: Support for dynamic validator set updates
- **Slashing Protection**: On-chain penalties for malicious behavior

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+
- Node.js 18+
- Foundry (for Ethereum contracts)
- Substrate development environment

### Installation
```bash
# Clone and setup
git clone <repository>
cd cross-chain-bridge

# Install dependencies
make install

# Run tests
make test

# Start local development environment
make dev
```

## 📊 Validator Configuration

The bridge operates with:
- **3 Ethereum validators** (2-of-3 threshold)
- **3 Polkadot validators** (2-of-3 threshold)
- **Configurable threshold parameters**

## 🧪 Testing

Comprehensive test suite includes:
- Unit tests for all components
- Integration tests for cross-chain flows
- Simulation with forked chains
- Validator rotation scenarios
- Security attack simulations

## 📚 Documentation

- [Architecture Design](docs/architecture.md)
- [Security Model](docs/security.md)
- [Deployment Guide](docs/deployment.md)
- [API Reference](docs/api.md)

## 🛡️ Security Considerations

This bridge implements multiple security layers:
- Threshold cryptography for validator consensus
- Event verification with cryptographic proofs
- Time-locked operations for emergency stops
- Comprehensive monitoring and alerting

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.
