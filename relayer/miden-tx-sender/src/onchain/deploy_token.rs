use miden_bridge::accounts::token_wrapper::TokenWrapperAccount;
use miden_client::account::component::{BasicFungibleFaucet};
use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::{Client, ClientError};
use miden_lib::account::auth::{AuthRpoFalcon512Acl, AuthRpoFalcon512AclConfig};
use miden_objects::account::{Account, AuthSecretKey};
use miden_objects::asset::{FungibleAsset, TokenSymbol};
use miden_objects::{Felt, Word, crypto::dsa::rpo_falcon512::SecretKey};
use rand::prelude::StdRng;
use rand::{rng, RngCore};

const MAX_SUPPLY: Felt = Felt::new(FungibleAsset::MAX_AMOUNT);

pub async fn insert_new_fungible_faucet(
    client: &mut Client<FilesystemKeyStore<StdRng>>,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore<StdRng>,
    symbol: &str,
    decimals: u8,
    origin_network: u64,
    origin_address: [Felt; 3],
) -> Result<(Account, Word), ClientError> {
    let mut rng = rng();

    let key_pair = SecretKey::with_rng(&mut rng);
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair)).unwrap();

    let mut init_seed = [0u8; 32];
    rng.fill_bytes(&mut init_seed);

    let symbol = TokenSymbol::new(symbol).unwrap();

    let (account, seed) = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(storage_mode)
        .with_auth_component(AuthRpoFalcon512Acl::new(
            pub_key,
            AuthRpoFalcon512AclConfig::new().with_auth_trigger_procedures(
                vec![BasicFungibleFaucet::distribute_digest()])
            )?
        )
        .with_component(TokenWrapperAccount::new(origin_network, origin_address))
        .with_component(BasicFungibleFaucet::new(symbol, decimals, MAX_SUPPLY).unwrap())
        .build()?;

    client.add_account(&account, Some(seed), false).await?;
    Ok((account, seed))
}
