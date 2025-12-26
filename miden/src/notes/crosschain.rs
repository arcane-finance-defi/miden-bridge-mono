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

pub struct CrosshainNoteParams {
    pub serial_number: Word,
    pub output_serial_number: Word,
    pub dest_chain: Felt,
    pub dest_addr: [Felt; 4],
    pub unblock_timestamp: Option<u32>,
    pub faucet_id: AccountId,
    pub asset_amount: u64,
    pub sender: AccountId,
    pub note_tag: NoteTag,
}

pub fn new_crosschain_note(params: CrosshainNoteParams) -> Result<Note, NoteError> {
    let note = Note::new(
        NoteAssets::new(vec![FungibleAsset::new(params.faucet_id, params.asset_amount)
            .map_err(NoteError::AddFungibleAssetBalanceError)?
            .into()])?,
        NoteMetadata::new(
            params.sender,
            NoteType::Private,
            params.note_tag,
            NoteExecutionHint::always(),
            Felt::ZERO,
        )?,
        NoteRecipient::new(
            params.serial_number,
            croschain(),
            NoteInputs::new(vec![
                params.output_serial_number[3],
                params.output_serial_number[2],
                params.output_serial_number[1],
                params.output_serial_number[0],
                params.dest_chain,
                params.dest_addr[3],
                params.dest_addr[2],
                params.dest_addr[1],
                params.dest_addr[0],
                Felt::new(params.unblock_timestamp.unwrap_or(0) as u64),
            ])?,
        ),
    );

    Ok(note)
}
