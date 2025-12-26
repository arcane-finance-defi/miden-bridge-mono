use miden_objects::{AssetError, TokenSymbolError};
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TokenMetadataError {
    #[error(transparent)]
    AssetParse(#[from] AssetError),
    #[error(transparent)]
    NumberOverflow(#[from] TryFromIntError),
    #[error(transparent)]
    TokenSymbol(#[from] TokenSymbolError),
}
