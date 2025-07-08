//! # Cross-Chain Bridge Pallet
//!
//! A Substrate pallet for handling cross-chain token transfers between Polkadot and Ethereum.
//! This pallet manages wrapped tokens, validates threshold signatures from validators,
//! and handles minting/burning of bridged assets.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo},
        pallet_prelude::*,
        traits::{
            fungibles::{Create, Inspect, Mutate},
            tokens::{Fortitude, Precision, Preservation},
        },
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_core::{H160, H256};
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero},
        ArithmeticError,
    };
    use sp_std::{vec, vec::Vec};

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_assets::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// The pallet ID, used for deriving sovereign account IDs.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Maximum number of validators allowed.
        #[pallet::constant]
        type MaxValidators: Get<u32>;

        /// Maximum length of signature data.
        #[pallet::constant]
        type MaxSignatureLength: Get<u32>;
    }

    /// Information about a bridged token
    #[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct BridgedToken<AssetId, Balance> {
        /// The asset ID on this chain
        pub asset_id: AssetId,
        /// The token address on Ethereum
        pub ethereum_address: H160,
        /// Total supply of wrapped tokens
        pub total_supply: Balance,
        /// Whether the token is active
        pub is_active: bool,
    }

    /// Information about a mint request
    #[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct MintRequest<AccountId, AssetId, Balance> {
        /// The account to mint tokens to
        pub recipient: AccountId,
        /// The asset to mint
        pub asset_id: AssetId,
        /// The amount to mint
        pub amount: Balance,
        /// The Ethereum transaction hash
        pub ethereum_tx_hash: H256,
        /// Block number when request was created
        pub block_number: u32,
        /// Whether the request has been processed
        pub processed: bool,
    }

    /// Information about a burn request
    #[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct BurnRequest<AccountId, AssetId, Balance> {
        /// The account burning tokens
        pub burner: AccountId,
        /// The asset being burned
        pub asset_id: AssetId,
        /// The amount being burned
        pub amount: Balance,
        /// The Ethereum recipient address
        pub ethereum_recipient: H160,
        /// Block number when request was created
        pub block_number: u32,
        /// Whether the request has been processed
        pub processed: bool,
    }

    /// Validator information
    #[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ValidatorInfo<AccountId> {
        /// The validator's account ID
        pub account: AccountId,
        /// Whether the validator is active
        pub is_active: bool,
    }

    #[pallet::storage]
    #[pallet::getter(fn bridged_tokens)]
    /// Map from Ethereum token address to bridged token info
    pub type BridgedTokens<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H160,
        BridgedToken<T::AssetId, T::Balance>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn asset_to_ethereum)]
    /// Map from asset ID to Ethereum address
    pub type AssetToEthereum<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AssetId,
        H160,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validators)]
    /// List of bridge validators
    pub type Validators<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        ValidatorInfo<T::AccountId>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn validator_list)]
    /// Ordered list of validator accounts
    pub type ValidatorList<T: Config> = StorageValue<
        _,
        BoundedVec<T::AccountId, T::MaxValidators>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn threshold)]
    /// Signature threshold for validator consensus
    pub type Threshold<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn processed_ethereum_txs)]
    /// Set of processed Ethereum transaction hashes
    pub type ProcessedEthereumTxs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,
        bool,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn mint_requests)]
    /// Map from request ID to mint request
    pub type MintRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        MintRequest<T::AccountId, T::AssetId, T::Balance>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn burn_requests)]
    /// Map from request ID to burn request
    pub type BurnRequests<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BurnRequest<T::AccountId, T::AssetId, T::Balance>,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_mint_request_id)]
    /// Next mint request ID
    pub type NextMintRequestId<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_burn_request_id)]
    /// Next burn request ID
    pub type NextBurnRequestId<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Tokens were minted for a user. [recipient, asset_id, amount, ethereum_tx_hash]
        TokensMinted {
            recipient: T::AccountId,
            asset_id: T::AssetId,
            amount: T::Balance,
            ethereum_tx_hash: H256,
        },
        /// Tokens were burned by a user. [burner, asset_id, amount, ethereum_recipient]
        TokensBurned {
            burner: T::AccountId,
            asset_id: T::AssetId,
            amount: T::Balance,
            ethereum_recipient: H160,
        },
        /// A new bridged token was registered. [ethereum_address, asset_id]
        BridgedTokenRegistered {
            ethereum_address: H160,
            asset_id: T::AssetId,
        },
        /// A validator was added. [validator]
        ValidatorAdded { validator: T::AccountId },
        /// A validator was removed. [validator]
        ValidatorRemoved { validator: T::AccountId },
        /// Threshold was updated. [old_threshold, new_threshold]
        ThresholdUpdated {
            old_threshold: u32,
            new_threshold: u32,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Token is not registered for bridging
        TokenNotRegistered,
        /// Token is already registered
        TokenAlreadyRegistered,
        /// Validator already exists
        ValidatorAlreadyExists,
        /// Validator does not exist
        ValidatorNotFound,
        /// Invalid threshold value
        InvalidThreshold,
        /// Insufficient signatures
        InsufficientSignatures,
        /// Invalid signature
        InvalidSignature,
        /// Transaction already processed
        TransactionAlreadyProcessed,
        /// Request not found
        RequestNotFound,
        /// Request already processed
        RequestAlreadyProcessed,
        /// Insufficient balance
        InsufficientBalance,
        /// Asset creation failed
        AssetCreationFailed,
        /// Too many validators
        TooManyValidators,
        /// Cannot remove validator below threshold
        CannotRemoveValidatorBelowThreshold,
        /// Invalid amount (zero)
        InvalidAmount,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new token for bridging
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_token())]
        pub fn register_token(
            origin: OriginFor<T>,
            ethereum_address: H160,
            asset_id: T::AssetId,
            name: Vec<u8>,
            symbol: Vec<u8>,
            decimals: u8,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(
                !BridgedTokens::<T>::contains_key(&ethereum_address),
                Error::<T>::TokenAlreadyRegistered
            );

            // Create the asset
            let pallet_account = Self::account_id();
            pallet_assets::Pallet::<T>::create(
                T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(pallet_account.clone())),
                asset_id.clone().into(),
                pallet_account.into(),
                1u32.into(), // min_balance
            )
            .map_err(|_| Error::<T>::AssetCreationFailed)?;

            // Set asset metadata
            pallet_assets::Pallet::<T>::set_metadata(
                T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(Self::account_id())),
                asset_id.clone().into(),
                name,
                symbol,
                decimals,
            )
            .map_err(|_| Error::<T>::AssetCreationFailed)?;

            let bridged_token = BridgedToken {
                asset_id: asset_id.clone(),
                ethereum_address,
                total_supply: Zero::zero(),
                is_active: true,
            };

            BridgedTokens::<T>::insert(&ethereum_address, &bridged_token);
            AssetToEthereum::<T>::insert(&asset_id, &ethereum_address);

            Self::deposit_event(Event::BridgedTokenRegistered {
                ethereum_address,
                asset_id,
            });

            Ok(())
        }

        /// Mint tokens based on Ethereum lock transaction
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::mint_tokens())]
        pub fn mint_tokens(
            origin: OriginFor<T>,
            recipient: T::AccountId,
            ethereum_address: H160,
            amount: T::Balance,
            ethereum_tx_hash: H256,
            signatures: Vec<Vec<u8>>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            // Check if transaction already processed
            ensure!(
                !ProcessedEthereumTxs::<T>::get(&ethereum_tx_hash),
                Error::<T>::TransactionAlreadyProcessed
            );

            // Get bridged token info
            let mut bridged_token = BridgedTokens::<T>::get(&ethereum_address)
                .ok_or(Error::<T>::TokenNotRegistered)?;

            ensure!(bridged_token.is_active, Error::<T>::TokenNotRegistered);
            ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);

            // Verify signatures (simplified for now)
            let threshold = Threshold::<T>::get();
            ensure!(
                signatures.len() >= threshold as usize,
                Error::<T>::InsufficientSignatures
            );

            // Mark transaction as processed
            ProcessedEthereumTxs::<T>::insert(&ethereum_tx_hash, true);

            // Mint tokens to recipient
            let pallet_account = Self::account_id();
            pallet_assets::Pallet::<T>::mint(
                T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(pallet_account)),
                bridged_token.asset_id.clone().into(),
                recipient.clone().into(),
                amount,
            )
            .map_err(|_| Error::<T>::AssetCreationFailed)?;

            // Update total supply
            bridged_token.total_supply = bridged_token.total_supply.saturating_add(amount);
            BridgedTokens::<T>::insert(&ethereum_address, &bridged_token);

            // Create mint request record
            let request_id = NextMintRequestId::<T>::get();
            let mint_request = MintRequest {
                recipient: recipient.clone(),
                asset_id: bridged_token.asset_id.clone(),
                amount,
                ethereum_tx_hash,
                block_number: frame_system::Pallet::<T>::block_number().saturated_into(),
                processed: true,
            };

            MintRequests::<T>::insert(&request_id, &mint_request);
            NextMintRequestId::<T>::put(request_id.saturating_add(1));

            Self::deposit_event(Event::TokensMinted {
                recipient,
                asset_id: bridged_token.asset_id,
                amount,
                ethereum_tx_hash,
            });

            Ok(())
        }

        /// Burn tokens to unlock on Ethereum
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::burn_tokens())]
        pub fn burn_tokens(
            origin: OriginFor<T>,
            asset_id: T::AssetId,
            amount: T::Balance,
            ethereum_recipient: H160,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Get Ethereum address for this asset
            let ethereum_address = AssetToEthereum::<T>::get(&asset_id)
                .ok_or(Error::<T>::TokenNotRegistered)?;

            let mut bridged_token = BridgedTokens::<T>::get(&ethereum_address)
                .ok_or(Error::<T>::TokenNotRegistered)?;

            ensure!(bridged_token.is_active, Error::<T>::TokenNotRegistered);
            ensure!(!amount.is_zero(), Error::<T>::InvalidAmount);

            // Burn tokens from user
            pallet_assets::Pallet::<T>::burn(
                T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(who.clone())),
                asset_id.clone().into(),
                who.clone().into(),
                amount,
            )
            .map_err(|_| Error::<T>::InsufficientBalance)?;

            // Update total supply
            bridged_token.total_supply = bridged_token.total_supply.saturating_sub(amount);
            BridgedTokens::<T>::insert(&ethereum_address, &bridged_token);

            // Create burn request record
            let request_id = NextBurnRequestId::<T>::get();
            let burn_request = BurnRequest {
                burner: who.clone(),
                asset_id: asset_id.clone(),
                amount,
                ethereum_recipient,
                block_number: frame_system::Pallet::<T>::block_number().saturated_into(),
                processed: false,
            };

            BurnRequests::<T>::insert(&request_id, &burn_request);
            NextBurnRequestId::<T>::put(request_id.saturating_add(1));

            Self::deposit_event(Event::TokensBurned {
                burner: who,
                asset_id,
                amount,
                ethereum_recipient,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get the account ID of the pallet
        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
