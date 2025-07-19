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

---

## 📜 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

The MIT License is a permissive license that allows for commercial use, modification, distribution, and private use. It only requires preservation of copyright and license notices.

## 🤝 Contributing

We welcome contributions from the community! Here's how you can get involved:

### Getting Started

1. **🍴 Fork the Repository**
   ```bash
   # Click the "Fork" button on GitHub or use GitHub CLI
   gh repo fork Subaskar-S/cross-chain-bridge
   ```

2. **📥 Clone Your Fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/cross-chain-bridge.git
   cd cross-chain-bridge
   ```

3. **🌿 Create a Feature Branch**
   ```bash
   # Create a descriptive branch name
   git checkout -b feature/add-new-chain-support
   # or
   git checkout -b fix/validator-rotation-bug
   ```

4. **🔧 Make Your Changes**
   - Follow the existing code style and conventions
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass: `cargo test --workspace`

5. **💾 Commit Your Changes**
   ```bash
   # Stage your changes
   git add .

   # Write a clear, descriptive commit message
   git commit -m "feat: add support for Binance Smart Chain integration

   - Implement BSC contract deployment scripts
   - Add BSC-specific event monitoring
   - Update threshold signature validation for BSC
   - Add comprehensive tests for BSC bridge operations"
   ```

6. **📤 Push to Your Fork**
   ```bash
   git push origin feature/add-new-chain-support
   ```

7. **🔄 Create a Pull Request**
   - Go to your fork on GitHub
   - Click "New Pull Request"
   - Fill out the PR template with:
     - Clear description of changes
     - Testing performed
     - Breaking changes (if any)
     - Screenshots (if UI changes)

### 📋 Contribution Guidelines

- **Code Quality**: Follow Rust best practices and run `cargo clippy`
- **Testing**: Maintain or improve test coverage
- **Documentation**: Update relevant docs for new features
- **Security**: Consider security implications of all changes
- **Performance**: Profile performance-critical changes

### 🐛 Reporting Issues

Found a bug? Please create an issue with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, etc.)

### 💡 Feature Requests

Have an idea? Open an issue with:
- Use case description
- Proposed solution
- Alternative approaches considered
- Implementation complexity estimate

## 👨‍💻 Made by

<div align="center">

**Subaskar_S**

[![GitHub](https://img.shields.io/badge/GitHub-100000?style=for-the-badge&logo=github&logoColor=white)](https://github.com/Subaskar-S)
[![LinkedIn](https://img.shields.io/badge/LinkedIn-0077B5?style=for-the-badge&logo=linkedin&logoColor=white)](https://www.linkedin.com/in/subaskar97)

*Blockchain Developer*

---

### 🌟 Connect with me:
- 💼 **Professional**: [LinkedIn Profile](https://www.linkedin.com/in/subaskar97)
- 🔧 **Code**: [GitHub Profile](https://github.com/Subaskar-S)
- 📧 **Email**: Available on GitHub profile
- 🌐 **Portfolio**: Check out my other blockchain projects!

</div>

---

<div align="center">

**⭐ If this project helped you, please give it a star! ⭐**

*Built with ❤️ for the decentralized future*

</div>
