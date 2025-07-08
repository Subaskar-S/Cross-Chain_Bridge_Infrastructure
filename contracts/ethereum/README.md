# Ethereum Smart Contracts

This directory contains the Ethereum side smart contracts for the cross-chain bridge.

## Contracts

### CrossChainBridge.sol
The main bridge contract that handles:
- Locking ERC20 tokens for cross-chain transfer
- Unlocking tokens based on Polkadot transaction proofs
- Validator management and threshold signature verification
- Emergency pause/unpause functionality

### TestToken.sol
A simple ERC20 token for testing the bridge functionality.

## Features

### Token Locking
- Users can lock supported ERC20 tokens to bridge them to Polkadot
- Emits `BridgeLock` events that validators monitor
- Supports multiple token types

### Token Unlocking
- Validators provide threshold signatures to unlock tokens
- Requires k-of-n validator consensus (configurable threshold)
- Prevents replay attacks using transaction hash tracking

### Validator Management
- Add/remove validators (owner only)
- Update signature threshold (owner only)
- Support for up to 100 validators

### Security Features
- Reentrancy protection
- Emergency pause functionality
- Signature verification with ECDSA
- Transaction replay prevention

## Setup

### Prerequisites
- [Foundry](https://getfoundry.sh/) installed
- Node.js for package management

### Installation
```bash
# Install Foundry dependencies
forge install

# Install OpenZeppelin contracts
forge install OpenZeppelin/openzeppelin-contracts
forge install OpenZeppelin/openzeppelin-contracts-upgradeable
forge install foundry-rs/forge-std
```

### Build
```bash
forge build
```

### Test
```bash
forge test
```

### Deploy

#### Local Development
```bash
# Start local Ethereum node (Anvil)
anvil

# Deploy to local network
forge script script/Deploy.s.sol --rpc-url local --broadcast
```

#### Testnet Deployment
```bash
# Set environment variables
export PRIVATE_KEY=your_private_key
export ETHERSCAN_API_KEY=your_etherscan_key

# Deploy to Goerli
forge script script/Deploy.s.sol --rpc-url goerli --broadcast --verify

# Deploy to Sepolia
forge script script/Deploy.s.sol --rpc-url sepolia --broadcast --verify
```

## Contract Addresses

Deployment addresses are saved in `deployments/latest.json` after deployment.

## Usage

### Locking Tokens
```solidity
// Approve tokens first
token.approve(bridgeAddress, amount);

// Lock tokens for bridging
bridge.lockTokens(tokenAddress, amount, polkadotRecipientAddress);
```

### Unlocking Tokens
```solidity
// Validators create signatures for the unlock message
bytes[] memory signatures = getValidatorSignatures();

// Unlock tokens
bridge.unlockTokens(
    userAddress,
    tokenAddress,
    amount,
    polkadotTxHash,
    signatures
);
```

## Events

### BridgeLock
Emitted when tokens are locked for bridging:
```solidity
event BridgeLock(
    address indexed user,
    address indexed token,
    uint256 amount,
    bytes32 indexed polkadotAddress,
    uint256 nonce
);
```

### BridgeUnlock
Emitted when tokens are unlocked from bridge:
```solidity
event BridgeUnlock(
    address indexed user,
    address indexed token,
    uint256 amount,
    bytes32 polkadotTxHash,
    uint256 nonce
);
```

## Security Considerations

1. **Validator Security**: Validators must secure their private keys
2. **Threshold Selection**: Choose appropriate k-of-n threshold for security vs availability
3. **Token Approval**: Users should only approve necessary amounts
4. **Emergency Procedures**: Owner can pause contract in emergencies
5. **Signature Verification**: All unlocks require valid threshold signatures

## Testing

The test suite covers:
- Token locking and unlocking flows
- Validator management
- Threshold signature verification
- Emergency pause functionality
- Access control
- Error conditions

Run tests with:
```bash
forge test -vvv
```

## Gas Optimization

The contracts are optimized for gas efficiency:
- Efficient signature verification
- Minimal storage operations
- Batch operations where possible

## Upgrades

The current contracts are not upgradeable. For production deployment, consider:
- Proxy patterns for upgradeability
- Timelock for admin operations
- Multi-sig for critical functions

## Integration

These contracts integrate with:
- Polkadot bridge pallet
- Rust relayer service
- Threshold signature system
- Monitoring and alerting systems
