use miden_bridge::utils::AddressFormatError;
use miden_client::ClientError;
use miden_client::rpc::RpcError;
use miden_client::store::StoreError;
use miden_client::transaction::{TransactionProverError, TransactionRequestError};
use miden_lib::account::faucets::FungibleFaucetError;
use miden_objects::{AccountError, AssetError, NoteError, TokenSymbolError};
use miden_objects::account::AccountId;
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
    #[error(transparent)]
    AddressFormatError(#[from] AddressFormatError),
    #[error(transparent)]
    TokenSymbolError(#[from] TokenSymbolError),
    #[error(transparent)]
    FungibleFaucetError(#[from] FungibleFaucetError),
    #[error("Account with id {0} not found in storage")]
    AccountNotFoundInStorage(AccountId),
}
