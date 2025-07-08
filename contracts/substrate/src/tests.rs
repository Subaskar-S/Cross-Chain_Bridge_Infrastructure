//! Tests for the cross-chain bridge pallet

use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok, traits::fungibles::Inspect};
use sp_core::{H160, H256};

#[test]
fn register_token_works() {
    new_test_ext().execute_with(|| {
        // Register a new token
        assert_ok!(CrossChainBridge::register_token(
            RuntimeOrigin::root(),
            ethereum_address(),
            asset_id(),
            token_name(),
            token_symbol(),
            token_decimals(),
        ));

        // Check that the token was registered
        let bridged_token = CrossChainBridge::bridged_tokens(ethereum_address()).unwrap();
        assert_eq!(bridged_token.asset_id, asset_id());
        assert_eq!(bridged_token.ethereum_address, ethereum_address());
        assert_eq!(bridged_token.total_supply, 0);
        assert!(bridged_token.is_active);

        // Check reverse mapping
        assert_eq!(
            CrossChainBridge::asset_to_ethereum(asset_id()),
            Some(ethereum_address())
        );

        // Check that event was emitted
        System::assert_last_event(
            Event::BridgedTokenRegistered {
                ethereum_address: ethereum_address(),
                asset_id: asset_id(),
            }
            .into(),
        );
    });
}

#[test]
fn register_token_fails_if_already_registered() {
    new_test_ext().execute_with(|| {
        // Register a token
        assert_ok!(CrossChainBridge::register_token(
            RuntimeOrigin::root(),
            ethereum_address(),
            asset_id(),
            token_name(),
            token_symbol(),
            token_decimals(),
        ));

        // Try to register the same token again
        assert_noop!(
            CrossChainBridge::register_token(
                RuntimeOrigin::root(),
                ethereum_address(),
                asset_id() + 1,
                token_name(),
                token_symbol(),
                token_decimals(),
            ),
            Error::<Test>::TokenAlreadyRegistered
        );
    });
}

#[test]
fn register_token_requires_root() {
    new_test_ext().execute_with(|| {
        // Try to register a token without root origin
        assert_noop!(
            CrossChainBridge::register_token(
                RuntimeOrigin::signed(1),
                ethereum_address(),
                asset_id(),
                token_name(),
                token_symbol(),
                token_decimals(),
            ),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn mint_tokens_works() {
    new_test_ext().execute_with(|| {
        // First register a token
        assert_ok!(CrossChainBridge::register_token(
            RuntimeOrigin::root(),
            ethereum_address(),
            asset_id(),
            token_name(),
            token_symbol(),
            token_decimals(),
        ));

        let recipient = 1u64;
        let amount = 1000u128;
        let signatures = vec![vec![1u8; 65], vec![2u8; 65]]; // Mock signatures

        // Set threshold to 2
        crate::Threshold::<Test>::put(2u32);

        // Mint tokens
        assert_ok!(CrossChainBridge::mint_tokens(
            RuntimeOrigin::signed(1),
            recipient,
            ethereum_address(),
            amount,
            ethereum_tx_hash(),
            signatures,
        ));

        // Check that tokens were minted
        assert_eq!(
            Assets::balance(asset_id(), &recipient),
            amount
        );

        // Check that transaction was marked as processed
        assert!(CrossChainBridge::processed_ethereum_txs(ethereum_tx_hash()));

        // Check that total supply was updated
        let bridged_token = CrossChainBridge::bridged_tokens(ethereum_address()).unwrap();
        assert_eq!(bridged_token.total_supply, amount);

        // Check that event was emitted
        System::assert_last_event(
            Event::TokensMinted {
                recipient,
                asset_id: asset_id(),
                amount,
                ethereum_tx_hash: ethereum_tx_hash(),
            }
            .into(),
        );
    });
}

#[test]
fn mint_tokens_fails_for_unregistered_token() {
    new_test_ext().execute_with(|| {
        let recipient = 1u64;
        let amount = 1000u128;
        let signatures = vec![vec![1u8; 65], vec![2u8; 65]];

        // Try to mint tokens for unregistered token
        assert_noop!(
            CrossChainBridge::mint_tokens(
                RuntimeOrigin::signed(1),
                recipient,
                ethereum_address(),
                amount,
                ethereum_tx_hash(),
                signatures,
            ),
            Error::<Test>::TokenNotRegistered
        );
    });
}

#[test]
fn mint_tokens_fails_for_processed_transaction() {
    new_test_ext().execute_with(|| {
        // Register a token
        assert_ok!(CrossChainBridge::register_token(
            RuntimeOrigin::root(),
            ethereum_address(),
            asset_id(),
            token_name(),
            token_symbol(),
            token_decimals(),
        ));

        let recipient = 1u64;
        let amount = 1000u128;
        let signatures = vec![vec![1u8; 65], vec![2u8; 65]];

        // Set threshold
        crate::Threshold::<Test>::put(2u32);

        // Mint tokens first time
        assert_ok!(CrossChainBridge::mint_tokens(
            RuntimeOrigin::signed(1),
            recipient,
            ethereum_address(),
            amount,
            ethereum_tx_hash(),
            signatures.clone(),
        ));

        // Try to mint tokens again with same transaction hash
        assert_noop!(
            CrossChainBridge::mint_tokens(
                RuntimeOrigin::signed(1),
                recipient,
                ethereum_address(),
                amount,
                ethereum_tx_hash(),
                signatures,
            ),
            Error::<Test>::TransactionAlreadyProcessed
        );
    });
}

#[test]
fn mint_tokens_fails_with_insufficient_signatures() {
    new_test_ext().execute_with(|| {
        // Register a token
        assert_ok!(CrossChainBridge::register_token(
            RuntimeOrigin::root(),
            ethereum_address(),
            asset_id(),
            token_name(),
            token_symbol(),
            token_decimals(),
        ));

        let recipient = 1u64;
        let amount = 1000u128;
        let signatures = vec![vec![1u8; 65]]; // Only one signature

        // Set threshold to 2
        crate::Threshold::<Test>::put(2u32);

        // Try to mint tokens with insufficient signatures
        assert_noop!(
            CrossChainBridge::mint_tokens(
                RuntimeOrigin::signed(1),
                recipient,
                ethereum_address(),
                amount,
                ethereum_tx_hash(),
                signatures,
            ),
            Error::<Test>::InsufficientSignatures
        );
    });
}

#[test]
fn burn_tokens_works() {
    new_test_ext().execute_with(|| {
        // Register a token and mint some tokens first
        assert_ok!(CrossChainBridge::register_token(
            RuntimeOrigin::root(),
            ethereum_address(),
            asset_id(),
            token_name(),
            token_symbol(),
            token_decimals(),
        ));

        let user = 1u64;
        let amount = 1000u128;
        let signatures = vec![vec![1u8; 65], vec![2u8; 65]];

        // Set threshold
        crate::Threshold::<Test>::put(2u32);

        // Mint tokens first
        assert_ok!(CrossChainBridge::mint_tokens(
            RuntimeOrigin::signed(1),
            user,
            ethereum_address(),
            amount,
            ethereum_tx_hash(),
            signatures,
        ));

        let burn_amount = 500u128;
        let ethereum_recipient = H160::from_slice(&[3u8; 20]);

        // Burn tokens
        assert_ok!(CrossChainBridge::burn_tokens(
            RuntimeOrigin::signed(user),
            asset_id(),
            burn_amount,
            ethereum_recipient,
        ));

        // Check that tokens were burned
        assert_eq!(
            Assets::balance(asset_id(), &user),
            amount - burn_amount
        );

        // Check that total supply was updated
        let bridged_token = CrossChainBridge::bridged_tokens(ethereum_address()).unwrap();
        assert_eq!(bridged_token.total_supply, amount - burn_amount);

        // Check that event was emitted
        System::assert_last_event(
            Event::TokensBurned {
                burner: user,
                asset_id: asset_id(),
                amount: burn_amount,
                ethereum_recipient,
            }
            .into(),
        );
    });
}

#[test]
fn burn_tokens_fails_for_unregistered_token() {
    new_test_ext().execute_with(|| {
        let user = 1u64;
        let amount = 1000u128;
        let ethereum_recipient = H160::from_slice(&[3u8; 20]);

        // Try to burn tokens for unregistered token
        assert_noop!(
            CrossChainBridge::burn_tokens(
                RuntimeOrigin::signed(user),
                asset_id(),
                amount,
                ethereum_recipient,
            ),
            Error::<Test>::TokenNotRegistered
        );
    });
}

#[test]
fn burn_tokens_fails_with_zero_amount() {
    new_test_ext().execute_with(|| {
        // Register a token
        assert_ok!(CrossChainBridge::register_token(
            RuntimeOrigin::root(),
            ethereum_address(),
            asset_id(),
            token_name(),
            token_symbol(),
            token_decimals(),
        ));

        let user = 1u64;
        let ethereum_recipient = H160::from_slice(&[3u8; 20]);

        // Try to burn zero tokens
        assert_noop!(
            CrossChainBridge::burn_tokens(
                RuntimeOrigin::signed(user),
                asset_id(),
                0u128,
                ethereum_recipient,
            ),
            Error::<Test>::InvalidAmount
        );
    });
}
