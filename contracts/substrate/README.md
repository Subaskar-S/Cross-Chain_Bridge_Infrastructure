# Cross-Chain Bridge Pallet

A Substrate pallet for handling cross-chain token transfers between Polkadot and Ethereum. This pallet manages wrapped tokens, validates threshold signatures from validators, and handles minting/burning of bridged assets.

## Overview

This pallet provides the Polkadot/Substrate side functionality for the cross-chain bridge. It works in conjunction with Ethereum smart contracts to enable secure token transfers between the two ecosystems.

## Features

### Token Management
- Register new tokens for cross-chain bridging
- Create wrapped asset representations of Ethereum tokens
- Track total supply and metadata for bridged tokens

### Minting Operations
- Mint wrapped tokens based on Ethereum lock transactions
- Validate threshold signatures from bridge validators
- Prevent replay attacks using transaction hash tracking

### Burning Operations
- Burn wrapped tokens to initiate unlock on Ethereum
- Emit events for validators to monitor and process
- Track burn requests for audit and monitoring

### Validator Management
- Add/remove bridge validators
- Configure signature thresholds
- Support for up to 100 validators

## Extrinsics

### `register_token`
Register a new Ethereum token for bridging.

**Parameters:**
- `ethereum_address`: The token contract address on Ethereum
- `asset_id`: The asset ID to use on Substrate
- `name`: Token name
- `symbol`: Token symbol
- `decimals`: Number of decimal places

**Origin:** Root

### `mint_tokens`
Mint wrapped tokens based on an Ethereum lock transaction.

**Parameters:**
- `recipient`: Account to receive the minted tokens
- `ethereum_address`: Ethereum token contract address
- `amount`: Amount of tokens to mint
- `ethereum_tx_hash`: Hash of the Ethereum lock transaction
- `signatures`: Array of validator signatures

**Origin:** Signed

### `burn_tokens`
Burn wrapped tokens to unlock on Ethereum.

**Parameters:**
- `asset_id`: The asset ID to burn
- `amount`: Amount of tokens to burn
- `ethereum_recipient`: Ethereum address to receive unlocked tokens

**Origin:** Signed

## Events

### `TokensMinted`
Emitted when tokens are minted for a user.
- `recipient`: Account that received tokens
- `asset_id`: Asset that was minted
- `amount`: Amount minted
- `ethereum_tx_hash`: Source Ethereum transaction

### `TokensBurned`
Emitted when tokens are burned by a user.
- `burner`: Account that burned tokens
- `asset_id`: Asset that was burned
- `amount`: Amount burned
- `ethereum_recipient`: Destination Ethereum address

### `BridgedTokenRegistered`
Emitted when a new token is registered for bridging.
- `ethereum_address`: Ethereum token contract address
- `asset_id`: Substrate asset ID

## Storage

### `BridgedTokens`
Map from Ethereum token address to bridged token information.

### `AssetToEthereum`
Reverse map from Substrate asset ID to Ethereum address.

### `Validators`
Map of validator accounts and their information.

### `ValidatorList`
Ordered list of active validators.

### `Threshold`
Current signature threshold for validator consensus.

### `ProcessedEthereumTxs`
Set of processed Ethereum transaction hashes to prevent replays.

### `MintRequests` / `BurnRequests`
Historical records of mint and burn operations.

## Configuration

### Required Traits
- `frame_system::Config`: Basic system functionality
- `pallet_assets::Config`: Asset management functionality

### Constants
- `PalletId`: Unique identifier for the pallet
- `MaxValidators`: Maximum number of validators (default: 100)
- `MaxSignatureLength`: Maximum signature length (default: 65 bytes)

## Security Features

### Signature Verification
- Threshold signature validation for mint operations
- Configurable k-of-n validator consensus
- Protection against signature replay attacks

### Transaction Replay Prevention
- Ethereum transaction hash tracking
- Prevents double-spending attacks
- Maintains audit trail of processed transactions

### Access Control
- Root-only token registration
- Validator management restricted to authorized accounts
- User-controlled burn operations

## Integration

### With Ethereum Contracts
- Monitors `BridgeLock` events from Ethereum
- Validates unlock signatures for Ethereum transactions
- Maintains consistency with Ethereum bridge state

### With Relayer Service
- Provides events for relayer monitoring
- Accepts validated signatures from relayers
- Coordinates cross-chain transaction flow

### With Threshold Signatures
- Integrates with threshold signature library
- Validates multi-party signatures
- Supports validator rotation and threshold updates

## Testing

The pallet includes comprehensive tests covering:
- Token registration and management
- Minting with signature validation
- Burning operations
- Error conditions and edge cases
- Access control verification

Run tests with:
```bash
cargo test
```

## Benchmarking

Weight functions are provided for all extrinsics:
- `register_token`: ~26ms
- `mint_tokens`: ~36ms  
- `burn_tokens`: ~31ms
- Validator operations: ~15-19ms

## Usage Example

```rust
// Register a new token (root only)
CrossChainBridge::register_token(
    RuntimeOrigin::root(),
    ethereum_token_address,
    asset_id,
    b"Wrapped ETH".to_vec(),
    b"WETH".to_vec(),
    18,
)?;

// Mint tokens (with validator signatures)
CrossChainBridge::mint_tokens(
    RuntimeOrigin::signed(relayer),
    recipient_account,
    ethereum_token_address,
    amount,
    ethereum_tx_hash,
    validator_signatures,
)?;

// Burn tokens (user initiated)
CrossChainBridge::burn_tokens(
    RuntimeOrigin::signed(user),
    asset_id,
    amount,
    ethereum_recipient_address,
)?;
```

## Future Enhancements

- Light client integration for trustless verification
- Automated validator rotation
- Fee collection mechanisms
- Cross-chain governance integration
- Support for NFTs and other asset types
