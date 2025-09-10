use miden_lib::{
    account::{auth::{AuthRpoFalcon512Acl, AuthRpoFalcon512AclConfig}, faucets::BasicFungibleFaucet},
    AuthScheme,
};
use miden_objects::{
    account::{
        Account, AccountBuilder, AccountComponent, AccountStorageMode, AccountType, StorageSlot,
    },
    asset::TokenSymbol,
    note::NoteTag,
    utils::sync::LazyLock,
    AccountError, Felt, Word,
};

use crate::accounts::components::token_wrapper_account_library;

const BRIDGE_TAG_USECASE: u16 = 12354;
const BRIDGE_TAG: LazyLock<NoteTag> =
    LazyLock::new(|| NoteTag::for_local_use_case(BRIDGE_TAG_USECASE, 0).unwrap());

pub fn bridge_note_tag() -> NoteTag {
    BRIDGE_TAG.clone()
}

pub struct TokenWrapperAccount {
    origin_network: u64,
    origin_address: [Felt; 3],
}

impl TokenWrapperAccount {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Creates a new [`BasicFungibleFaucet`] component from the given pieces of metadata.
    pub fn new(origin_network: u64, origin_address: [Felt; 3]) -> Self {
        Self { origin_network, origin_address }
    }
}

impl From<TokenWrapperAccount> for AccountComponent {
    fn from(faucet: TokenWrapperAccount) -> Self {
        AccountComponent::new(
            token_wrapper_account_library(),
            vec![
                StorageSlot::Value(Word::new([
                    Felt::new(faucet.origin_network),
                    faucet.origin_address[2],
                    faucet.origin_address[1],
                    faucet.origin_address[0],
                ]))
            ]).expect("basic fungible faucet component should satisfy the requirements of a valid account component")
                .with_supported_type(AccountType::FungibleFaucet)
        }

}

fn builder_internal(
    init_seed: [u8; 32],
    symbol: TokenSymbol,
    decimals: u8,
    max_supply: Felt,
    origin_network: u64,
    origin_address: [Felt; 3],
    account_storage_mode: AccountStorageMode,
) -> Result<AccountBuilder, AccountError> {
    Ok(AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(account_storage_mode)
        .with_component(TokenWrapperAccount::new(origin_network, origin_address))
        .with_component(
            BasicFungibleFaucet::new(symbol, decimals, max_supply)
                .expect("Fungible faucet component build failed"),
        ))
}

pub fn create_token_wrapper_account(
    init_seed: [u8; 32],
    symbol: TokenSymbol,
    decimals: u8,
    max_supply: Felt,
    origin_network: u64,
    origin_address: [Felt; 3],
    account_storage_mode: AccountStorageMode,
    auth_scheme: AuthScheme,
) -> Result<(Account, Word), AccountError> {
    let auth_component: AuthRpoFalcon512Acl = match auth_scheme {
        AuthScheme::RpoFalcon512 { pub_key } => {
            Ok(AuthRpoFalcon512Acl::new(pub_key, AuthRpoFalcon512AclConfig::new().with_auth_trigger_procedures(
                vec![BasicFungibleFaucet::distribute_digest()]
            ))?)
        },
        _ => {
            Err(AccountError::other("unsupported auth scheme"))
        }
    }?;

    let (account, account_seed) = builder_internal(
        init_seed,
        symbol,
        decimals,
        max_supply,
        origin_network,
        origin_address,
        account_storage_mode,
    )?
    .with_auth_component(auth_component)
    .build()?;

    Ok((account, account_seed))
}

#[cfg(any(feature = "testing", test))]
pub fn create_token_wrapper_account_builder(
    init_seed: [u8; 32],
    symbol: TokenSymbol,
    decimals: u8,
    max_supply: Felt,
    origin_network: u64,
    origin_address: [Felt; 3],
    account_storage_mode: AccountStorageMode,
) -> Result<AccountBuilder, AccountError> {
    builder_internal(
        init_seed,
        symbol,
        decimals,
        max_supply,
        origin_network,
        origin_address,
        account_storage_mode,
    )
}
