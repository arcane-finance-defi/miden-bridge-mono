use miden_objects::Word;
use miden_objects::asset::TokenSymbol;
use miden_objects::utils::ToHex;
use crate::utils::errors::TokenMetadataError;

pub fn decode_slot_into_token_metadata(slot: Word) -> Result<(TokenSymbol, u8), TokenMetadataError> {
    let [_max_supply, decimals, symbol, _] = slot;

    Ok((
        TokenSymbol::try_from(symbol)?,
        u8::try_from(decimals.as_int())?
    ))
}

mod tests {
    use miden_objects::utils::parse_hex_string_as_word;
    use miden_objects::Word;
    use super::decode_slot_into_token_metadata;

    #[test]
    fn should_decode_slot_value() {
        let slot: Word = parse_hex_string_as_word("0xfeffffff00000000060000000000000013340000000000000000000000000000")
            .expect("Hex decoding to word failed");
        let (symbol, decimals) = decode_slot_into_token_metadata(slot).unwrap();
        assert_eq!(decimals, 6);
        assert_eq!(symbol.to_string(), Ok("AAATST".to_string()))
    }
}