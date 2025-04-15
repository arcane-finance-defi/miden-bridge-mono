use miden_client::ClientError;
use miden_client::rpc::RpcError;
use miden_client::store::StoreError;
use miden_client::transaction::{TransactionProverError, TransactionRequestError};
use miden_objects::{AccountError, AssetError, NoteError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OnchainError {
    #[error(transparent)]
    RpcCallError(#[from] RpcError),
    #[error(transparent)]
    AccountError(#[from] AccountError),
    #[error(transparent)]
    AssetError(#[from] AssetError),
    #[error(transparent)]
    NoteError(#[from] NoteError),
    #[error(transparent)]
    MidenClientError(#[from] ClientError),
    #[error(transparent)]
    TransactionProverError(#[from] TransactionProverError),
    #[error(transparent)]
    TransactionBuilderError(#[from] TransactionRequestError),
    #[error(transparent)]
    StoreError(#[from] StoreError),
}
