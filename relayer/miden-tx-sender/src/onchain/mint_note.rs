use miden_client::transaction::TransactionRequestBuilder;
use miden_crypto::Word;
use miden_objects::account::Account;
use miden_objects::asset::{Asset, FungibleAsset};
use miden_objects::Felt;
use miden_objects::note::{NoteAssets, NoteExecutionHint, NoteMetadata, NoteTag, NoteType, PartialNote};
use miden_objects::transaction::OutputNote;
use crate::onchain::client::OnchainClient;
use crate::onchain::CreatedTokenAccount;
use crate::onchain::errors::OnchainError;

const BRIDGE_USECASE: u16 = 15593;

pub async fn mint_asset(
    client: &mut OnchainClient,
    created_token_account: CreatedTokenAccount,
    recipient: Word,
    amount: u64,
) -> Result<Account, OnchainError> {

    let asset = Asset::Fungible(
        FungibleAsset::new(created_token_account.account().id(), amount).map_err(OnchainError::from)?,
    );

    let assets = NoteAssets::new(vec![asset]).map_err(OnchainError::from)?;

    let tx_request = TransactionRequestBuilder::new().with_own_output_notes(vec![
        OutputNote::Partial(PartialNote::new(
            NoteMetadata::new(
                created_token_account.account().id(),
                NoteType::Private,
                NoteTag::for_local_use_case(BRIDGE_USECASE, 0).map_err(OnchainError::from)?,
                NoteExecutionHint::Always,
                Felt::new(0)
            ).map_err(OnchainError::from)?,
            recipient.into(),
            assets
        ))
    ]).map_err(OnchainError::from)?.build();

    let delta = client.execute_tx(
        tx_request,
        created_token_account.account().id(),
        created_token_account.auth_secret_key()
    ).await?;

    created_token_account.account().apply_delta(&delta);

    Ok(created_token_account.account())
}