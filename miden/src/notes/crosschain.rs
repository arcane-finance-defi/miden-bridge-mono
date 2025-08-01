use miden_objects::{
    account::AccountId,
    asset::FungibleAsset,
    note::{
        Note, NoteAssets, NoteExecutionHint, NoteInputs, NoteMetadata, NoteRecipient, NoteTag,
        NoteType,
    },
    Felt, FieldElement, NoteError, Word,
};

use super::bridge::croschain;

pub fn new_crosschain_note(
    serial_number: Word,
    output_serial_number: Word,
    dest_chain: Felt,
    dest_addr: [Felt; 3],
    faucet_id: AccountId,
    asset_amount: u64,
    sender: AccountId,
    note_tag: NoteTag,
) -> Result<Note, NoteError> {
    let note = Note::new(
        NoteAssets::new(vec![FungibleAsset::new(faucet_id, asset_amount)
            .map_err(|e| NoteError::AddFungibleAssetBalanceError(e))?
            .into()])?,
        NoteMetadata::new(
            sender,
            NoteType::Private,
            note_tag,
            NoteExecutionHint::always(),
            Felt::ZERO,
        )?,
        NoteRecipient::new(
            serial_number,
            croschain(),
            NoteInputs::new(vec![
                output_serial_number[3],
                output_serial_number[2],
                output_serial_number[1],
                output_serial_number[0],
                dest_chain,
                dest_addr[2],
                dest_addr[1],
                dest_addr[0],
                Felt::ZERO,
                Felt::ZERO,
                Felt::ZERO,
                Felt::ZERO,
            ])?,
        ),
    );

    Ok(note)
}
