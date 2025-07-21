use crate::onchain::client::OnchainClient;
use crate::onchain::errors::OnchainError;
use crate::onchain::CreatedTokenAccount;
use miden_bridge::accounts::token_wrapper::TokenWrapperAccount;
use miden_client::account::component::{BasicFungibleFaucet, RpoFalcon512};
use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::{Client, ClientError};
use miden_crypto::{hash::rpo::Rpo256};
use miden_objects::account::{Account, AuthSecretKey};
use miden_objects::asset::TokenSymbol;
use miden_objects::{Felt, Word, crypto::dsa::rpo_falcon512::SecretKey};
use rand::prelude::StdRng;
use rand::{rng, RngCore};

const MAX_SUPPLY: Felt = Felt::new(u64::MAX);

pub async fn insert_new_fungible_faucet(
    client: &mut Client,
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
        .with_auth_component(RpoFalcon512::new(pub_key))
        .with_component(TokenWrapperAccount::new(origin_network, origin_address))
        .with_component(BasicFungibleFaucet::new(symbol, decimals, MAX_SUPPLY).unwrap())
        .build()?;

    client.add_account(&account, Some(seed), false).await?;
    Ok((account, seed))
}

fn calculate_init_seed(origin_network: u32, origin_address: [u8; 20]) -> [u8; 32] {
    let preimage = [origin_network.to_le_bytes().as_slice(), origin_address.as_slice()].concat();
    Rpo256::hash(preimage.as_slice()).into()
}

pub async fn deploy(
    client: &mut OnchainClient,
    origin_network: u32,
    origin_address: [u8; 20],
    symbol: &str,
    decimals: u8,
) -> Result<CreatedTokenAccount, OnchainError> {
    let init_seed = calculate_init_seed(origin_network, origin_address);

    let key_pair = SecretKey::new();

    let builder = AccountBuilder::new(init_seed)
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Public)
        .with_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(
            BasicFungibleFaucet::new(
                TokenSymbol::new(symbol).map_err(OnchainError::TokenSymbolError)?,
                decimals,
                MAX_SUPPLY,
            )
            .map_err(OnchainError::FungibleFaucetError)?,
        );

    let (new_account, seed) = builder.build().map_err(OnchainError::AccountError)?;

    Ok(CreatedTokenAccount::new(
        new_account,
        seed,
        AuthSecretKey::RpoFalcon512(key_pair),
    ))
}
