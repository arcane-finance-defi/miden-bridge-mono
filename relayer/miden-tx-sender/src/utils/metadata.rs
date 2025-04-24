use miden_crypto::Word;
use miden_objects::asset::TokenSymbol;
use miden_objects::utils::ToHex;
use crate::utils::errors::TokenMetadataError;

pub fn decode_slot_into_token_metadata(slot: Word) -> Result<(TokenSymbol, u8), TokenMetadataError> {
    let [_max_supply, decimals, symbol, _] = slot;

    Ok((
        TokenSymbol::try_from(symbol)
            .map_err(TokenMetadataError::from)?,
        u8::try_from(decimals.as_int())
            .map_err(TokenMetadataError::from)?
    ))
}

mod tests {
    use miden_objects::utils::parse_hex_string_as_word;
    use super::decode_slot_into_token_metadata;

    #[test]
    fn should_decode_slot_value() {
        let slot = parse_hex_string_as_word("0xfeffffff00000000060000000000000013340000000000000000000000000000").unwrap();
        let (symbol, decimals) = decode_slot_into_token_metadata(slot).unwrap();
        assert_eq!(decimals, 6);
        assert_eq!(symbol.to_str(), "AAATST".to_string())
    }
}