use std::fmt;
use std::num::TryFromIntError;
use alloy_primitives::hex;
use miden_client::utils::DeserializationError;
use miden_objects::{AssetError, TokenSymbolError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddressFormatError {
    #[error(transparent)]
    MalformedEvmAddress(#[from] hex::FromHexError),
    #[error(transparent)]
    FeltDeserializationError(#[from] DeserializationError),
    #[error(transparent)]
    FmtError(#[from] fmt::Error),
}

#[derive(Error, Debug)]
pub enum TokenMetadataError {
    #[error(transparent)]
    AssetParseError(#[from] AssetError),
    #[error(transparent)]
    NumberOverflowError(#[from] TryFromIntError),
    #[error(transparent)]
    TokenSymbolError(#[from] TokenSymbolError),
}