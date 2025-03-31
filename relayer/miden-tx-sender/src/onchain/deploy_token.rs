use miden_client::account::{AccountBuilder, AccountStorageMode, AccountType};
use miden_client::account::component::{BasicFungibleFaucet, RpoFalcon512};
use miden_crypto::{hash::rpo::Rpo256, dsa::rpo_falcon512::SecretKey };
use miden_objects::account::AuthSecretKey;
use miden_objects::asset::TokenSymbol;
use miden_objects::Felt;
use crate::onchain::client::OnchainClient;
use crate::onchain::CreatedTokenAccount;
use crate::onchain::errors::OnchainError;

const MAX_SUPPLY: Felt = Felt::new(u64::MAX);

fn calculate_init_seed(origin_network: u32, origin_address: [u8; 20]) -> [u8; 32] {
    let preimage = [origin_network.to_le_bytes().as_slice(), origin_address.as_slice()].concat();
    Rpo256::hash(preimage.as_slice()).into()
}

pub async fn deploy(
    client: &mut OnchainClient,
    origin_network: u32,
    origin_address: [u8; 20],
    symbol: &str,
    decimals: u8
) -> Result<CreatedTokenAccount, OnchainError> {
    let init_seed = calculate_init_seed(origin_network, origin_address);

    let key_pair = SecretKey::new();

    let anchor_block = client.get_anchor_block().await?;

    let builder = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().expect("anchor block should be valid"))
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(AccountStorageMode::Public)
        .with_component(RpoFalcon512::new(key_pair.public_key()))
        .with_component(BasicFungibleFaucet::new(
            TokenSymbol::new(symbol).map_err(OnchainError::from)?,
            decimals,
            MAX_SUPPLY
        ).map_err(OnchainError::from)?);


    let (new_account, seed) = builder
        .build()
        .map_err(OnchainError::from)?;

    Ok(CreatedTokenAccount::new(
        new_account,
        seed,
        AuthSecretKey::RpoFalcon512(key_pair)
    ))
}