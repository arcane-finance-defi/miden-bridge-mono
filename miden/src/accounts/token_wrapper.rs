use miden_lib::account::auth::RpoFalcon512;
use miden_lib::account::faucets::BasicFungibleFaucet;
use miden_lib::AuthScheme;
use crate::accounts::components::token_wrapper_account_library;

use miden_objects::asset::TokenSymbol;
use miden_objects::{AccountError, Felt, Word};
use miden_objects::account::{Account, AccountBuilder, AccountComponent, AccountIdAnchor, AccountStorageMode, AccountType};
use miden_objects::note::NoteTag;
use miden_objects::utils::sync::LazyLock;

const BRIDGE_TAG_USECASE: u16 = 12354;
const BRIDGE_TAG: LazyLock<NoteTag> = LazyLock::new(|| NoteTag::for_local_use_case(
    BRIDGE_TAG_USECASE, 0
).unwrap());

pub fn bridge_note_tag() -> NoteTag {
    BRIDGE_TAG.clone()
}

pub struct TokenWrapperAccount {
}

impl TokenWrapperAccount {

    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------

    /// Creates a new [`BasicFungibleFaucet`] component from the given pieces of metadata.
    pub fn new() -> Self {

        Self { }
    }
}


impl From<TokenWrapperAccount> for AccountComponent {
    fn from(_faucet: TokenWrapperAccount) -> Self {

        AccountComponent::new(token_wrapper_account_library(), vec![])
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
        .with_component(TokenWrapperAccount::new())
        .with_component(BasicFungibleFaucet::new(symbol, decimals, max_supply)?)
        .build()?;

    Ok((account, account_seed))
}