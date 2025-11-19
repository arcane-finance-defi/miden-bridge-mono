use crate::utils::errors::TokenMetadataError;
use miden_objects::Word;
use miden_objects::asset::TokenSymbol;

pub fn decode_slot_into_token_metadata(
    slot: Word,
) -> Result<(TokenSymbol, u8), TokenMetadataError> {
    let [_max_supply, decimals, symbol, _] = slot.each_ref();

    Ok((TokenSymbol::try_from(symbol.clone())?, u8::try_from(decimals.as_int())?))
}

#[cfg(test)]
mod tests {
    use super::Word;
    use super::decode_slot_into_token_metadata;
    #[test]
    fn should_decode_slot_value() {
        let slot =
            Word::parse("0xfeffffff00000000060000000000000013340000000000000000000000000000")
                .expect("Hex decoding to word failed")
                .into();
        let (symbol, decimals) = decode_slot_into_token_metadata(slot).unwrap();
        assert_eq!(decimals, 6);
        eprintln!("{symbol:?}");
        assert_eq!(symbol.to_string().expect("symbol to string"), "AAATST".to_string())
    }
}
