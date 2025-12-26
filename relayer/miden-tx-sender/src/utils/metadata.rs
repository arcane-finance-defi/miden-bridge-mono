use crate::utils::errors::TokenMetadataError;
use miden_objects::Word;
use miden_objects::asset::TokenSymbol;

pub fn decode_slot_into_token_metadata(
    slot: Word,
) -> Result<(TokenSymbol, u8), TokenMetadataError> {
    let [_max_supply, decimals, symbol, _x] = slot.each_ref();
    let symbol = TokenSymbol::try_from(*symbol)?;
    let decimals = u8::try_from(decimals.as_int())?;
    Ok((symbol, decimals))
}

#[cfg(test)]
mod tests {
    use super::Word;
    use super::decode_slot_into_token_metadata;
    #[test]
    fn should_decode_slot_value() {
        let slot =
            Word::parse("0xfeffffff000000000600000000000000F4490500000000000000000000000000")
                .expect("Hex decoding to word failed")
                .into();
        let (symbol, decimals) = decode_slot_into_token_metadata(slot).unwrap();
        assert_eq!(decimals, 6);
        assert_eq!(symbol.to_string().expect("symbol to string"), "AAATST".to_string())
    }
}
