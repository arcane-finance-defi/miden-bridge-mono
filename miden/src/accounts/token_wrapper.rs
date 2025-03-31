use miden_lib::account::auth::RpoFalcon512;
use miden_lib::AuthScheme;
use crate::accounts::components::token_wrapper_account_library;

use miden_objects::asset::TokenSymbol;
use miden_objects::{AccountError, Felt, FieldElement, Word};
use miden_objects::account::{Account, AccountBuilder, AccountComponent, AccountIdAnchor, AccountStorageMode, AccountType, StorageSlot};

pub struct TokenWrapperAccount {
    symbol: TokenSymbol,
    decimals: u8,
    max_supply: Felt,
}

impl TokenWrapperAccount {
    const MAX_MAX_SUPPLY: u64 = (1 << 63) - 1;
    const MAX_DECIMALS: u8 = 12;

    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Creates a new [`BasicFungibleFaucet`] component from the given pieces of metadata.
    pub fn new(symbol: TokenSymbol, decimals: u8, max_supply: Felt) -> Result<Self, AccountError> {
        // First check that the metadata is valid.
        if decimals > Self::MAX_DECIMALS {
            return Err(AccountError::FungibleFaucetTooManyDecimals {
                actual: decimals,
                max: Self::MAX_DECIMALS,
            });
        } else if max_supply.as_int() > Self::MAX_MAX_SUPPLY {
            return Err(AccountError::FungibleFaucetMaxSupplyTooLarge {
                actual: max_supply.as_int(),
                max: Self::MAX_MAX_SUPPLY,
            });
        }

        Ok(Self { symbol, decimals, max_supply })
    }
}


impl From<TokenWrapperAccount> for AccountComponent {
    fn from(faucet: TokenWrapperAccount) -> Self {
        // Note: data is stored as [a0, a1, a2, a3] but loaded onto the stack as
        // [a3, a2, a1, a0, ...]
        let metadata =
            [faucet.max_supply, Felt::from(faucet.decimals), faucet.symbol.into(), Felt::ZERO];

        AccountComponent::new(token_wrapper_account_library(), vec![StorageSlot::Value(metadata)])
            .expect("basic fungible faucet component should satisfy the requirements of a valid account component")
            .with_supported_type(AccountType::FungibleFaucet)
    }
}

pub fn create_token_wrapper_account(
    init_seed: [u8; 32],
    id_anchor: AccountIdAnchor,
    symbol: TokenSymbol,
    decimals: u8,
    max_supply: Felt,
    account_storage_mode: AccountStorageMode,
    auth_scheme: AuthScheme,
) -> Result<(Account, Word), AccountError> {
    let auth_component: RpoFalcon512 = match auth_scheme {
        AuthScheme::RpoFalcon512 { pub_key } => RpoFalcon512::new(pub_key),
    };

    let (account, account_seed) = AccountBuilder::new(init_seed)
        .anchor(id_anchor)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(account_storage_mode)
        .with_component(auth_component)
        .with_component(TokenWrapperAccount::new(symbol, decimals, max_supply)?)
        .build()?;

    Ok((account, account_seed))
}