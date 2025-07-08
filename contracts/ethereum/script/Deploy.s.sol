// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/CrossChainBridge.sol";
import "../src/TestToken.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        console.log("Deploying contracts with account:", deployer);
        console.log("Account balance:", deployer.balance);

        vm.startBroadcast(deployerPrivateKey);

        // Deploy test token
        TestToken testToken = new TestToken(
            "Bridge Test Token",
            "BTT",
            18,
            1000000 // 1M tokens
        );
        console.log("TestToken deployed at:", address(testToken));

        // Setup validators (for testing - in production these would be real validator addresses)
        address[] memory validators = new address[](3);
        validators[0] = 0x70997970C51812dc3A010C7d01b50e0d17dc79C8; // Hardhat account 1
        validators[1] = 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC; // Hardhat account 2
        validators[2] = 0x90F79bf6EB2c4f870365E785982E1f101E93b906; // Hardhat account 3

        // Setup supported tokens
        address[] memory supportedTokens = new address[](1);
        supportedTokens[0] = address(testToken);

        // Deploy bridge with 2-of-3 threshold
        CrossChainBridge bridge = new CrossChainBridge(
            validators,
            2, // threshold
            supportedTokens
        );
        console.log("CrossChainBridge deployed at:", address(bridge));

        // Verify deployment
        console.log("Bridge threshold:", bridge.threshold());
        console.log("Bridge validator count:", bridge.getValidatorCount());
        console.log("Test token supported:", bridge.supportedTokens(address(testToken)));

        vm.stopBroadcast();

        // Save deployment addresses to file
        string memory deploymentInfo = string(
            abi.encodePacked(
                "{\n",
                '  "CrossChainBridge": "', vm.toString(address(bridge)), '",\n',
                '  "TestToken": "', vm.toString(address(testToken)), '",\n',
                '  "deployer": "', vm.toString(deployer), '",\n',
                '  "chainId": ', vm.toString(block.chainid), ',\n',
                '  "blockNumber": ', vm.toString(block.number), '\n',
                "}"
            )
        );

        vm.writeFile("./deployments/latest.json", deploymentInfo);
        console.log("Deployment info saved to ./deployments/latest.json");
    }
}
