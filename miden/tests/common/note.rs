use miden_assembly::Report;
use miden_lib::transaction::TransactionKernel;
use miden_objects::asset::FungibleAsset;
use miden_objects::note::{Note, NoteAssets, NoteInputs, NoteMetadata, NoteRecipient, NoteScript, NoteType};
use miden_objects::{Felt, FieldElement, Word};
use miden_objects::account::AccountId;
use miden_objects::testing::account_id::ACCOUNT_ID_SENDER;
use miden_bridge::full_library;
//
// pub fn get_p2id_crosschain_note(
//     asset: FungibleAsset,
//     serial_num: Word,
//
// )

pub fn get_note_with_fungible_asset_and_script(
    fungible_asset: FungibleAsset,
    note_script: String,
) -> Result<Note, Report> {
    use miden_objects::note::NoteExecutionHint;

    let assembler = TransactionKernel::testing_assembler()
        .with_library(full_library())?.with_debug_mode(true);

    let note_script = NoteScript::compile(note_script, assembler).unwrap();
    const SERIAL_NUM: Word = [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];
    let sender_id = AccountId::try_from(ACCOUNT_ID_SENDER).unwrap();

    let vault = NoteAssets::new(vec![fungible_asset.into()]).unwrap();
    let metadata =
        NoteMetadata::new(sender_id, NoteType::Public, 1.into(), NoteExecutionHint::Always, Felt::ZERO)
            .unwrap();
    let inputs = NoteInputs::new(vec![]).unwrap();
    let recipient = NoteRecipient::new(SERIAL_NUM, note_script, inputs);

    Ok(Note::new(vault, metadata, recipient))
}