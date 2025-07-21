use miden_bridge::utils::{felts_to_evm_addresses, AddressFormatError};
use miden_objects::Word;
use miden_objects::utils::ToHex;

pub fn decode_slot_into_origin_info(slot: Word) -> Result<(u32, String), AddressFormatError> {
    let (origin_network, origin_address) = slot
        .split_at_checked(1)
        .unwrap();
    let origin_address = felts_to_evm_addresses(
        [
            origin_address[2],
            origin_address[1],
            origin_address[0]
        ]
    )?;

    Ok((origin_network[0].as_int().try_into().unwrap(), origin_address.to_checksum(None)))
}

mod tests {
    use miden_objects::utils::parse_hex_string_as_word;
    use super::decode_slot_into_origin_info;

    #[test]
    fn should_decode_slot_value() {
        let slot = parse_hex_string_as_word("0x8238010000000000fd9ae61e000000008e784c5a1efa36822f476def8a5e8141").unwrap();
        let (origin_network, origin_address) = decode_slot_into_origin_info(slot).unwrap();
        assert_eq!(origin_network, 80002);
        assert_eq!(origin_address.to_lowercase(), "0x2f476def8a5e81418e784c5a1efa3682fd9ae61e".to_string())
    }
}