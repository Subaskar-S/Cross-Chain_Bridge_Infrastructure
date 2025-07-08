// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/**
 * @title CrossChainBridge
 * @dev Ethereum side of the cross-chain bridge for token transfers to Polkadot
 * @notice This contract handles locking/unlocking of ERC20 tokens and validates threshold signatures
 */
contract CrossChainBridge is ReentrancyGuard, Pausable, Ownable {
    using SafeERC20 for IERC20;
    using ECDSA for bytes32;

    // Events
    event BridgeLock(
        address indexed user,
        address indexed token,
        uint256 amount,
        bytes32 indexed polkadotAddress,
        uint256 nonce
    );

    event BridgeUnlock(
        address indexed user,
        address indexed token,
        uint256 amount,
        bytes32 polkadotTxHash,
        uint256 nonce
    );

    event ValidatorAdded(address indexed validator, uint256 newThreshold);
    event ValidatorRemoved(address indexed validator, uint256 newThreshold);
    event ThresholdUpdated(uint256 oldThreshold, uint256 newThreshold);
    event EmergencyPause(address indexed admin);
    event EmergencyUnpause(address indexed admin);

    // Structs
    struct LockRequest {
        address user;
        address token;
        uint256 amount;
        bytes32 polkadotAddress;
        uint256 timestamp;
        bool processed;
    }

    struct UnlockRequest {
        address user;
        address token;
        uint256 amount;
        bytes32 polkadotTxHash;
        uint256 timestamp;
        bool processed;
    }

    // State variables
    mapping(address => bool) public validators;
    mapping(bytes32 => bool) public processedTxHashes;
    mapping(uint256 => LockRequest) public lockRequests;
    mapping(uint256 => UnlockRequest) public unlockRequests;
    mapping(address => bool) public supportedTokens;
    
    address[] public validatorList;
    uint256 public threshold;
    uint256 public nonce;
    uint256 public lockNonce;
    uint256 public unlockNonce;
    
    // Constants
    uint256 public constant MAX_VALIDATORS = 100;
    uint256 public constant MIN_THRESHOLD = 1;
    uint256 public constant SIGNATURE_VALIDITY_PERIOD = 1 hours;

    // Modifiers
    modifier onlyValidator() {
        require(validators[msg.sender], "Not a validator");
        _;
    }

    modifier validThreshold(uint256 _threshold) {
        require(_threshold >= MIN_THRESHOLD, "Threshold too low");
        require(_threshold <= validatorList.length, "Threshold too high");
        _;
    }

    constructor(
        address[] memory _validators,
        uint256 _threshold,
        address[] memory _supportedTokens
    ) validThreshold(_threshold) {
        require(_validators.length > 0, "No validators provided");
        require(_validators.length <= MAX_VALIDATORS, "Too many validators");

        // Add validators
        for (uint256 i = 0; i < _validators.length; i++) {
            require(_validators[i] != address(0), "Invalid validator address");
            require(!validators[_validators[i]], "Duplicate validator");
            
            validators[_validators[i]] = true;
            validatorList.push(_validators[i]);
        }

        threshold = _threshold;

        // Add supported tokens
        for (uint256 i = 0; i < _supportedTokens.length; i++) {
            require(_supportedTokens[i] != address(0), "Invalid token address");
            supportedTokens[_supportedTokens[i]] = true;
        }
    }

    /**
     * @dev Lock tokens to be bridged to Polkadot
     * @param token The ERC20 token address to lock
     * @param amount The amount of tokens to lock
     * @param polkadotAddress The recipient address on Polkadot (32 bytes)
     */
    function lockTokens(
        address token,
        uint256 amount,
        bytes32 polkadotAddress
    ) external nonReentrant whenNotPaused {
        require(supportedTokens[token], "Token not supported");
        require(amount > 0, "Amount must be greater than 0");
        require(polkadotAddress != bytes32(0), "Invalid Polkadot address");

        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        uint256 currentNonce = ++lockNonce;
        
        lockRequests[currentNonce] = LockRequest({
            user: msg.sender,
            token: token,
            amount: amount,
            polkadotAddress: polkadotAddress,
            timestamp: block.timestamp,
            processed: false
        });

        emit BridgeLock(msg.sender, token, amount, polkadotAddress, currentNonce);
    }

    /**
     * @dev Unlock tokens from Polkadot bridge transaction
     * @param user The user address to receive tokens
     * @param token The ERC20 token address to unlock
     * @param amount The amount of tokens to unlock
     * @param polkadotTxHash The transaction hash from Polkadot
     * @param signatures Array of validator signatures
     */
    function unlockTokens(
        address user,
        address token,
        uint256 amount,
        bytes32 polkadotTxHash,
        bytes[] calldata signatures
    ) external nonReentrant whenNotPaused {
        require(user != address(0), "Invalid user address");
        require(supportedTokens[token], "Token not supported");
        require(amount > 0, "Amount must be greater than 0");
        require(!processedTxHashes[polkadotTxHash], "Transaction already processed");
        require(signatures.length >= threshold, "Insufficient signatures");

        // Verify signatures
        bytes32 messageHash = keccak256(
            abi.encodePacked(user, token, amount, polkadotTxHash, block.chainid)
        );
        bytes32 ethSignedMessageHash = messageHash.toEthSignedMessageHash();

        _verifySignatures(ethSignedMessageHash, signatures);

        // Mark transaction as processed
        processedTxHashes[polkadotTxHash] = true;

        uint256 currentNonce = ++unlockNonce;
        
        unlockRequests[currentNonce] = UnlockRequest({
            user: user,
            token: token,
            amount: amount,
            polkadotTxHash: polkadotTxHash,
            timestamp: block.timestamp,
            processed: true
        });

        // Transfer tokens to user
        IERC20(token).safeTransfer(user, amount);

        emit BridgeUnlock(user, token, amount, polkadotTxHash, currentNonce);
    }

    /**
     * @dev Verify threshold signatures from validators
     * @param messageHash The hash of the message that was signed
     * @param signatures Array of signatures to verify
     */
    function _verifySignatures(
        bytes32 messageHash,
        bytes[] calldata signatures
    ) internal view {
        address[] memory signers = new address[](signatures.length);
        uint256 validSignatures = 0;

        for (uint256 i = 0; i < signatures.length; i++) {
            address signer = messageHash.recover(signatures[i]);
            
            // Check if signer is a validator and not already counted
            if (validators[signer] && !_contains(signers, signer, validSignatures)) {
                signers[validSignatures] = signer;
                validSignatures++;
            }
        }

        require(validSignatures >= threshold, "Insufficient valid signatures");
    }

    /**
     * @dev Check if an address is already in the signers array
     */
    function _contains(
        address[] memory signers,
        address signer,
        uint256 length
    ) internal pure returns (bool) {
        for (uint256 i = 0; i < length; i++) {
            if (signers[i] == signer) {
                return true;
            }
        }
        return false;
    }

    /**
     * @dev Add a new validator (only owner)
     * @param validator The validator address to add
     */
    function addValidator(address validator) external onlyOwner {
        require(validator != address(0), "Invalid validator address");
        require(!validators[validator], "Validator already exists");
        require(validatorList.length < MAX_VALIDATORS, "Too many validators");

        validators[validator] = true;
        validatorList.push(validator);

        emit ValidatorAdded(validator, threshold);
    }

    /**
     * @dev Remove a validator (only owner)
     * @param validator The validator address to remove
     */
    function removeValidator(address validator) external onlyOwner {
        require(validators[validator], "Validator does not exist");
        require(validatorList.length > threshold, "Cannot remove validator below threshold");

        validators[validator] = false;

        // Remove from validator list
        for (uint256 i = 0; i < validatorList.length; i++) {
            if (validatorList[i] == validator) {
                validatorList[i] = validatorList[validatorList.length - 1];
                validatorList.pop();
                break;
            }
        }

        emit ValidatorRemoved(validator, threshold);
    }

    /**
     * @dev Update the signature threshold (only owner)
     * @param newThreshold The new threshold value
     */
    function updateThreshold(uint256 newThreshold) external onlyOwner validThreshold(newThreshold) {
        uint256 oldThreshold = threshold;
        threshold = newThreshold;
        emit ThresholdUpdated(oldThreshold, newThreshold);
    }

    /**
     * @dev Add support for a new token (only owner)
     * @param token The token address to support
     */
    function addSupportedToken(address token) external onlyOwner {
        require(token != address(0), "Invalid token address");
        require(!supportedTokens[token], "Token already supported");
        
        supportedTokens[token] = true;
    }

    /**
     * @dev Remove support for a token (only owner)
     * @param token The token address to remove support for
     */
    function removeSupportedToken(address token) external onlyOwner {
        require(supportedTokens[token], "Token not supported");
        supportedTokens[token] = false;
    }

    /**
     * @dev Emergency pause (only owner)
     */
    function emergencyPause() external onlyOwner {
        _pause();
        emit EmergencyPause(msg.sender);
    }

    /**
     * @dev Emergency unpause (only owner)
     */
    function emergencyUnpause() external onlyOwner {
        _unpause();
        emit EmergencyUnpause(msg.sender);
    }

    /**
     * @dev Get the list of validators
     */
    function getValidators() external view returns (address[] memory) {
        return validatorList;
    }

    /**
     * @dev Get validator count
     */
    function getValidatorCount() external view returns (uint256) {
        return validatorList.length;
    }

    /**
     * @dev Check if a transaction hash has been processed
     */
    function isTransactionProcessed(bytes32 txHash) external view returns (bool) {
        return processedTxHashes[txHash];
    }

    /**
     * @dev Get lock request details
     */
    function getLockRequest(uint256 _nonce) external view returns (LockRequest memory) {
        return lockRequests[_nonce];
    }

    /**
     * @dev Get unlock request details
     */
    function getUnlockRequest(uint256 _nonce) external view returns (UnlockRequest memory) {
        return unlockRequests[_nonce];
    }
}
