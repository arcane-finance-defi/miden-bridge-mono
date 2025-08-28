use core::fmt;

use alloy_primitives::{
    hex::{FromHex, FromHexError},
    Address,
};
use miden_objects::{
    utils::{DeserializationError, Serializable},
    Felt, StarkField,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddressFormatError {
    #[error(transparent)]
    MalformedEvmAddress(#[from] FromHexError),
    #[error(transparent)]
    FeltDeserializationError(#[from] DeserializationError),
    #[error(transparent)]
    FmtError(#[from] fmt::Error),
}

pub fn evm_address_to_felts<T>(address: T) -> Result<[Felt; 3], AddressFormatError>
where
    T: AsRef<str>,
{
    let evm_dest_address =
        Address::from_hex(address.as_ref()).map_err(AddressFormatError::MalformedEvmAddress)?;

    let address_felts = [
        Felt::try_from(&evm_dest_address.0[..8])
            .map_err(AddressFormatError::FeltDeserializationError)?,
        Felt::try_from(&evm_dest_address.0[8..16])
            .map_err(AddressFormatError::FeltDeserializationError)?,
        Felt::from_bytes_with_padding(&evm_dest_address.0[16..20]),
    ];

    Ok(address_felts)
}

pub fn felts_to_evm_addresses(felts: [Felt; 3]) -> Result<Address, AddressFormatError> {
    let address_bytes =
        vec![felts[0].to_bytes(), felts[1].to_bytes(), felts[2].to_bytes()[..4].to_vec()].concat();

    let evm_dest_address = Address::from_slice(address_bytes.as_slice());

    Ok(evm_dest_address)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use miden_objects::utils::ToHex;

    use super::{evm_address_to_felts, felts_to_evm_addresses};

    #[test]
    fn should_decode_encoded_evm_address() {
        let inputs = [
            "0xAB348CB6A2Bf1aE152C793e091ff0545cF0Ad7b7",
            "0x20b0bad0c3C9C3f40A88801A5E8e24043B9c6C10",
            "0xA69FD3dB73147241E129EAd5B8F06C4F89E43D37",
            "0x5d3326797595DBEDa89a1BEc498D4A3DBf4A2cC2",
        ];

        for input in inputs {
            let felts = evm_address_to_felts(input.to_string()).unwrap();
            let output = felts_to_evm_addresses(felts).unwrap();
            let hex_output = output.to_hex_with_prefix();
            assert_eq!(input.to_lowercase(), hex_output.to_lowercase().as_str());
        }
    }
}
