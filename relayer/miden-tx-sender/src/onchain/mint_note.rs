use crate::onchain::client::execute_tx;
use crate::onchain::errors::OnchainError;
use miden_client::Client;
use miden_client::transaction::{TransactionRequestBuilder, TransactionResult};
use miden_crypto::Word;
use miden_objects::Felt;
use miden_objects::account::AccountId;
use miden_objects::asset::{Asset as MidenAsset, FungibleAsset};
use miden_objects::note::{
    NoteAssets, NoteExecutionHint, NoteMetadata, NoteTag, NoteType, PartialNote,
};
use miden_objects::transaction::OutputNote;
use serde::{Deserialize, Serialize};

const BRIDGE_USECASE: u16 = 15593;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct Asset {
    pub origin_network: u32,
    pub origin_address: String,
    pub asset_symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MintArgs {
    pub asset: Asset,
    pub amount: u64,
    pub recipient: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct MintedNote {
    pub note_id: String,
    pub faucet_id: String,
    pub transaction_id: String,
}

pub async fn mint_asset(
    client: &mut Client,
    faucet_id: AccountId,
    recipient: Word,
    amount: u64,
) -> Result<TransactionResult, OnchainError> {
    let asset =
        MidenAsset::Fungible(FungibleAsset::new(faucet_id, amount).map_err(OnchainError::from)?);

    let assets = NoteAssets::new(vec![asset]).map_err(OnchainError::from)?;

    let tx_request = TransactionRequestBuilder::new()
        .with_own_output_notes(vec![OutputNote::Partial(PartialNote::new(
            NoteMetadata::new(
                faucet_id,
                NoteType::Private,
                NoteTag::for_local_use_case(BRIDGE_USECASE, 0).map_err(OnchainError::from)?,
                NoteExecutionHint::Always,
                Felt::new(0),
            )
            .map_err(OnchainError::from)?,
            recipient.into(),
            assets,
        ))])
        .build()?;

    let transaction =
        execute_tx(client, tx_request, faucet_id).await.map_err(OnchainError::from)?;

    Ok(transaction)
}

pub async fn mint_fungible_asset(
    execution_client: &mut Client,
    faucet_id: AccountId,
    recipient: AccountId,
    amount: u64,
) -> Result<TransactionResult, OnchainError> {
    let tx_request = TransactionRequestBuilder::mint_fungible_asset(
        FungibleAsset::new(faucet_id, amount)?,
        recipient,
        NoteType::Private,
        execution_client.rng(),
    )?
    .build()?;

    let transaction = execute_tx(execution_client, tx_request, faucet_id).await?;

    Ok(transaction)
}
