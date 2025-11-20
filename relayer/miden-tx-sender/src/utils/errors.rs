use miden_objects::{AssetError, TokenSymbolError};
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenMetadataError {
    #[error(transparent)]
    AssetParseError(#[from] AssetError),
    #[error(transparent)]
    NumberOverflowError(#[from] TryFromIntError),
    #[error(transparent)]
    TokenSymbolError(#[from] TokenSymbolError),
}
