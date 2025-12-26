use core::{array::TryFromSliceError, fmt};

use alloy_primitives::{
    hex::{FromHex, FromHexError},
    Address,
};
use miden_objects::{
    utils::{DeserializationError, Serializable},
    Felt, StarkField,
};
use solana_address::Address as SolanaAddress;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddressFormatError {
    #[error(transparent)]
    MalformedEvmAddress(#[from] FromHexError),
    #[error(transparent)]
    FeltDeserializationError(#[from] DeserializationError),
    #[error(transparent)]
    FmtError(#[from] fmt::Error),
    #[error("Solana decoding error: {0}")]
    MalformedSolanaAddress(TryFromSliceError),
}

pub fn evm_address_to_felts<T>(address: T) -> Result<[Felt; 4], AddressFormatError>
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
        Felt::new(0),
    ];

    Ok(address_felts)
}

pub fn felts_to_evm_addresses(felts: [Felt; 4]) -> Result<Address, AddressFormatError> {
    let address_bytes =
        [felts[0].to_bytes(), felts[1].to_bytes(), felts[2].to_bytes()[..4].to_vec()].concat();

    let evm_dest_address = Address::from_slice(address_bytes.as_slice());

    Ok(evm_dest_address)
}

pub fn solana_address_to_felts<T>(address: T) -> Result<[Felt; 4], AddressFormatError>
where
    T: AsRef<str>,
{
    let dest_address = SolanaAddress::from_str_const(address.as_ref()).to_bytes();

    let address_felts = [
        Felt::try_from(&dest_address[..8]).map_err(AddressFormatError::FeltDeserializationError)?,
        Felt::try_from(&dest_address[8..16])
            .map_err(AddressFormatError::FeltDeserializationError)?,
        Felt::try_from(&dest_address[16..24])
            .map_err(AddressFormatError::FeltDeserializationError)?,
        Felt::try_from(&dest_address[24..32])
            .map_err(AddressFormatError::FeltDeserializationError)?,
    ];

    Ok(address_felts)
}

pub fn felts_to_solana_addresses(felts: [Felt; 4]) -> Result<SolanaAddress, AddressFormatError> {
    let address_bytes = [
        felts[0].to_bytes(),
        felts[1].to_bytes(),
        felts[2].to_bytes(),
        felts[3].to_bytes(),
    ]
    .concat();

    let dest_address = SolanaAddress::new_from_array(
        address_bytes
            .as_slice()
            .try_into()
            .map_err(AddressFormatError::MalformedSolanaAddress)?,
    );

    Ok(dest_address)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use miden_objects::utils::ToHex;

    use super::{
        evm_address_to_felts, felts_to_evm_addresses, felts_to_solana_addresses,
        solana_address_to_felts,
    };

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

    #[test]
    fn should_decode_encoded_solana_address() {
        let inputs = [
            "G1XMT99oDxSsYd4KvTcutCTmSGL2h8d1KDV7B1oWy7Qv",
            "Cbn3Z3Zj67ZLNb4Vz6VuYG3Kxeym6An4j5UEXmCiPVg7",
            "DFjHao5TBrKeovaXuGRqLDeh6xxUanB6Vtz6cmbmGnkF",
            "CkeqapGjB9qinNo9HbZvYL52fa5ykiyvWaXMsm9rnf8X",
        ];

        for input in inputs {
            let felts = solana_address_to_felts(input.to_string()).unwrap();
            let output = felts_to_solana_addresses(felts).unwrap();
            let output = output.to_string();
            assert_eq!(input.to_lowercase(), output.to_lowercase().as_str());
        }
    }
}
