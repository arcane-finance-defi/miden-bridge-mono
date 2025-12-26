use miden_bridge::utils::{AddressFormatError, felts_to_evm_addresses};
use miden_objects::Word;

pub fn decode_slots_into_origin_info(address_slot: Word, chain_slot: Word) -> Result<(u32, String), AddressFormatError> {
    let origin_address =
        felts_to_evm_addresses([address_slot[3], address_slot[2], address_slot[1], address_slot[0]])?;

    Ok((chain_slot[0].as_int().try_into().unwrap(), origin_address.to_checksum(None)))
}

#[cfg(test)]
mod tests {
    use super::Word;
    use super::decode_slots_into_origin_info;
    #[test]
    fn should_decode_slot_value() {
        let address_slot =
            Word::parse("0x0000000000000000fd9ae61e000000008e784c5a1efa36822f476def8a5e8141")
                .unwrap();
        let chain_slot =
            Word::parse("0x8238010000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        let (origin_network, origin_address) = decode_slots_into_origin_info(address_slot, chain_slot).unwrap();
        assert_eq!(origin_network, 80002);
        assert_eq!(
            origin_address.to_lowercase(),
            "0x2f476def8a5e81418e784c5a1efa3682fd9ae61e".to_string()
        )
    }
}
