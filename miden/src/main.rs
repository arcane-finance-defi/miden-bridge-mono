use core::fmt;
use std::{env, fs::File, io::Write, num::ParseIntError};

use miden_lib::note::utils::build_p2id_recipient;
use miden_objects::{
    account::AccountId,
    asset::{Asset, FungibleAsset},
    crypto::{
        rand::{FeltRng, RpoRandomCoin},
        utils::word_to_hex,
    },
    note::{
        Note, NoteAssets, NoteDetails, NoteExecutionHint, NoteFile, NoteMetadata, NoteTag, NoteType,
    },
    utils::{parse_hex_string_as_word, Serializable},
    AccountIdError, AssetError, Felt, FieldElement, NoteError, Word,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    ParseAccountIdError(#[from] AccountIdError),
    #[error(transparent)]
    BuildNoteRecipientError(NoteError),
    #[error(transparent)]
    BuildExportableNoteError(NoteError),
    #[error(transparent)]
    WordHexEncodingError(#[from] fmt::Error),
    #[error("Hex parsing error: `{0}`")]
    BytesHexDecodingError(String),
    #[error(transparent)]
    InvalidAmountError(#[from] ParseIntError),
    #[error(transparent)]
    FungibleAssetBuildError(#[from] AssetError),
    #[error("Unknown command: `{0}`")]
    UnknownCommandError(String),
    #[error("Invalid note save path")]
    InvalidSavePathError(),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn main() -> Result<(), CliError> {
    let args: Vec<String> = env::args().collect();

    let cmd = &args[1];

    if cmd == "generate" {
        build_recipient(args[2].clone())
    } else if cmd == "reconstruct" {
        restore_note(args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone())
    } else {
        Err(CliError::UnknownCommandError(cmd.clone()))
    }
}

fn build_recipient(receiver: String) -> Result<(), CliError> {
    let mut rng = RpoRandomCoin::new([Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)]);

    let serial_number: Word = rng.draw_word();
    let receiver =
        AccountId::from_hex(&receiver.as_str()).map_err(|e| CliError::ParseAccountIdError(e))?;

    let recipient = build_p2id_recipient(receiver, serial_number.clone())
        .map_err(|e| CliError::BuildNoteRecipientError(e))?;

    let recipient_digest = recipient.digest().to_hex();
    let serial_number =
        word_to_hex(&serial_number).map_err(|e| CliError::WordHexEncodingError(e))?;

    println!("Recipient: {recipient_digest}");
    println!("Serial number: {serial_number}");

    Ok(())
}

fn restore_note(
    receiver: String,
    serial_num_hex: String,
    bridged_amount: String,
    faucet_id: String,
) -> Result<(), CliError> {
    let serial_number = parse_hex_string_as_word(&serial_num_hex)
        .map_err(|e| CliError::BytesHexDecodingError(e.parse().unwrap()))?;

    let receiver =
        AccountId::from_hex(&receiver.as_str()).map_err(|e| CliError::ParseAccountIdError(e))?;

    let recipient = build_p2id_recipient(receiver, serial_number.clone())
        .map_err(|e| CliError::BuildNoteRecipientError(e))?;

    let faucet_id =
        AccountId::from_hex(&faucet_id.as_str()).map_err(|e| CliError::ParseAccountIdError(e))?;

    let bridged_amount = u64::from_str_radix(bridged_amount.as_str(), 10)
        .map_err(|e| CliError::InvalidAmountError(e))?;

    let asset = FungibleAsset::new(faucet_id, bridged_amount)
        .map_err(|e| CliError::FungibleAssetBuildError(e))?;

    let note = Note::new(
        NoteAssets::new(vec![Asset::from(asset)])
            .map_err(|e| CliError::BuildExportableNoteError(e))?,
        NoteMetadata::new(
            faucet_id,
            NoteType::Private,
            NoteTag::for_local_use_case(1, 0).map_err(|e| CliError::BuildExportableNoteError(e))?,
            NoteExecutionHint::Always,
            Felt::ZERO,
        )
        .map_err(|e| CliError::BuildExportableNoteError(e))?,
        recipient,
    );

    let note_id = note.clone().id().to_hex();

    println!("Reconstructed note id: {note_id}");

    let note_details = NoteDetails::new(note.assets().clone(), note.recipient().clone());

    const BRIDGE_USECASE: u16 = 14594;

    let note_text = NoteFile::NoteDetails {
        details: note_details,
        after_block_num: 0.into(),
        tag: Some(
            NoteTag::for_local_use_case(BRIDGE_USECASE, 0)
                .map_err(|e| CliError::BuildExportableNoteError(e))?,
        ),
    };

    let file_path = env::current_dir()
        .map_err(|_| CliError::InvalidSavePathError())?
        .join("reconstructed_note.mno");

    let mut file = File::create(file_path).map_err(CliError::from)?;
    file.write_all(&note_text.to_bytes()).map_err(CliError::from)?;

    Ok(())
}
