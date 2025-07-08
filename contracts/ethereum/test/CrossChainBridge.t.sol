// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/CrossChainBridge.sol";
import "../src/TestToken.sol";

contract CrossChainBridgeTest is Test {
    CrossChainBridge public bridge;
    TestToken public token;
    
    address public owner;
    address public user1;
    address public user2;
    address public validator1;
    address public validator2;
    address public validator3;
    
    address[] public validators;
    address[] public supportedTokens;
    
    uint256 public constant INITIAL_SUPPLY = 1000000 * 10**18;
    uint256 public constant THRESHOLD = 2;
    
    bytes32 public constant POLKADOT_ADDRESS = bytes32(uint256(0x1234567890abcdef));
    bytes32 public constant POLKADOT_TX_HASH = bytes32(uint256(0xabcdef1234567890));

    function setUp() public {
        owner = address(this);
        user1 = makeAddr("user1");
        user2 = makeAddr("user2");
        validator1 = makeAddr("validator1");
        validator2 = makeAddr("validator2");
        validator3 = makeAddr("validator3");
        
        // Deploy test token
        token = new TestToken("Test Token", "TEST", 18, INITIAL_SUPPLY);
        
        // Setup validators
        validators.push(validator1);
        validators.push(validator2);
        validators.push(validator3);
        
        // Setup supported tokens
        supportedTokens.push(address(token));
        
        // Deploy bridge
        bridge = new CrossChainBridge(validators, THRESHOLD, supportedTokens);
        
        // Give tokens to users
        token.transfer(user1, 10000 * 10**18);
        token.transfer(user2, 10000 * 10**18);
    }

    function testInitialState() public {
        assertEq(bridge.threshold(), THRESHOLD);
        assertEq(bridge.getValidatorCount(), 3);
        assertTrue(bridge.validators(validator1));
        assertTrue(bridge.validators(validator2));
        assertTrue(bridge.validators(validator3));
        assertTrue(bridge.supportedTokens(address(token)));
    }

    function testLockTokens() public {
        uint256 amount = 1000 * 10**18;
        
        vm.startPrank(user1);
        token.approve(address(bridge), amount);
        
        vm.expectEmit(true, true, true, true);
        emit CrossChainBridge.BridgeLock(user1, address(token), amount, POLKADOT_ADDRESS, 1);
        
        bridge.lockTokens(address(token), amount, POLKADOT_ADDRESS);
        vm.stopPrank();
        
        // Check balances
        assertEq(token.balanceOf(user1), 9000 * 10**18);
        assertEq(token.balanceOf(address(bridge)), amount);
        
        // Check lock request
        CrossChainBridge.LockRequest memory request = bridge.getLockRequest(1);
        assertEq(request.user, user1);
        assertEq(request.token, address(token));
        assertEq(request.amount, amount);
        assertEq(request.polkadotAddress, POLKADOT_ADDRESS);
        assertFalse(request.processed);
    }

    function testLockTokensFailures() public {
        uint256 amount = 1000 * 10**18;
        
        vm.startPrank(user1);
        
        // Test unsupported token
        TestToken unsupportedToken = new TestToken("Unsupported", "UNSUP", 18, 1000);
        vm.expectRevert("Token not supported");
        bridge.lockTokens(address(unsupportedToken), amount, POLKADOT_ADDRESS);
        
        // Test zero amount
        vm.expectRevert("Amount must be greater than 0");
        bridge.lockTokens(address(token), 0, POLKADOT_ADDRESS);
        
        // Test zero Polkadot address
        vm.expectRevert("Invalid Polkadot address");
        bridge.lockTokens(address(token), amount, bytes32(0));
        
        vm.stopPrank();
    }

    function testUnlockTokensWithValidSignatures() public {
        uint256 amount = 1000 * 10**18;
        
        // First, lock some tokens to have balance in bridge
        vm.startPrank(user1);
        token.approve(address(bridge), amount);
        bridge.lockTokens(address(token), amount, POLKADOT_ADDRESS);
        vm.stopPrank();
        
        // Create message hash
        bytes32 messageHash = keccak256(
            abi.encodePacked(user2, address(token), amount, POLKADOT_TX_HASH, block.chainid)
        );
        bytes32 ethSignedMessageHash = messageHash.toEthSignedMessageHash();
        
        // Create signatures from validators
        bytes[] memory signatures = new bytes[](2);
        (uint8 v1, bytes32 r1, bytes32 s1) = vm.sign(uint256(keccak256(abi.encodePacked("validator1"))), ethSignedMessageHash);
        (uint8 v2, bytes32 r2, bytes32 s2) = vm.sign(uint256(keccak256(abi.encodePacked("validator2"))), ethSignedMessageHash);
        
        signatures[0] = abi.encodePacked(r1, s1, v1);
        signatures[1] = abi.encodePacked(r2, s2, v2);
        
        uint256 initialBalance = token.balanceOf(user2);
        
        vm.expectEmit(true, true, true, true);
        emit CrossChainBridge.BridgeUnlock(user2, address(token), amount, POLKADOT_TX_HASH, 1);
        
        bridge.unlockTokens(user2, address(token), amount, POLKADOT_TX_HASH, signatures);
        
        // Check balances
        assertEq(token.balanceOf(user2), initialBalance + amount);
        assertTrue(bridge.isTransactionProcessed(POLKADOT_TX_HASH));
        
        // Check unlock request
        CrossChainBridge.UnlockRequest memory request = bridge.getUnlockRequest(1);
        assertEq(request.user, user2);
        assertEq(request.token, address(token));
        assertEq(request.amount, amount);
        assertEq(request.polkadotTxHash, POLKADOT_TX_HASH);
        assertTrue(request.processed);
    }

    function testUnlockTokensFailures() public {
        uint256 amount = 1000 * 10**18;
        bytes[] memory signatures = new bytes[](1); // Insufficient signatures
        
        // Test insufficient signatures
        vm.expectRevert("Insufficient signatures");
        bridge.unlockTokens(user2, address(token), amount, POLKADOT_TX_HASH, signatures);
        
        // Test invalid user address
        signatures = new bytes[](2);
        vm.expectRevert("Invalid user address");
        bridge.unlockTokens(address(0), address(token), amount, POLKADOT_TX_HASH, signatures);
        
        // Test unsupported token
        TestToken unsupportedToken = new TestToken("Unsupported", "UNSUP", 18, 1000);
        vm.expectRevert("Token not supported");
        bridge.unlockTokens(user2, address(unsupportedToken), amount, POLKADOT_TX_HASH, signatures);
        
        // Test zero amount
        vm.expectRevert("Amount must be greater than 0");
        bridge.unlockTokens(user2, address(token), 0, POLKADOT_TX_HASH, signatures);
    }

    function testValidatorManagement() public {
        address newValidator = makeAddr("newValidator");
        
        // Add validator
        bridge.addValidator(newValidator);
        assertTrue(bridge.validators(newValidator));
        assertEq(bridge.getValidatorCount(), 4);
        
        // Remove validator
        bridge.removeValidator(newValidator);
        assertFalse(bridge.validators(newValidator));
        assertEq(bridge.getValidatorCount(), 3);
    }

    function testThresholdUpdate() public {
        uint256 newThreshold = 3;
        
        vm.expectEmit(true, true, false, true);
        emit CrossChainBridge.ThresholdUpdated(THRESHOLD, newThreshold);
        
        bridge.updateThreshold(newThreshold);
        assertEq(bridge.threshold(), newThreshold);
    }

    function testTokenManagement() public {
        TestToken newToken = new TestToken("New Token", "NEW", 18, 1000);
        
        // Add supported token
        bridge.addSupportedToken(address(newToken));
        assertTrue(bridge.supportedTokens(address(newToken)));
        
        // Remove supported token
        bridge.removeSupportedToken(address(newToken));
        assertFalse(bridge.supportedTokens(address(newToken)));
    }

    function testPauseUnpause() public {
        // Pause
        bridge.emergencyPause();
        assertTrue(bridge.paused());
        
        // Try to lock tokens while paused
        vm.startPrank(user1);
        token.approve(address(bridge), 1000 * 10**18);
        vm.expectRevert("Pausable: paused");
        bridge.lockTokens(address(token), 1000 * 10**18, POLKADOT_ADDRESS);
        vm.stopPrank();
        
        // Unpause
        bridge.emergencyUnpause();
        assertFalse(bridge.paused());
    }

    function testOnlyOwnerFunctions() public {
        vm.startPrank(user1);
        
        vm.expectRevert("Ownable: caller is not the owner");
        bridge.addValidator(makeAddr("newValidator"));
        
        vm.expectRevert("Ownable: caller is not the owner");
        bridge.removeValidator(validator1);
        
        vm.expectRevert("Ownable: caller is not the owner");
        bridge.updateThreshold(3);
        
        vm.expectRevert("Ownable: caller is not the owner");
        bridge.emergencyPause();
        
        vm.stopPrank();
    }

    // Helper function to create ECDSA signature
    function signMessage(uint256 privateKey, bytes32 messageHash) internal pure returns (bytes memory) {
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privateKey, messageHash);
        return abi.encodePacked(r, s, v);
    }
}
