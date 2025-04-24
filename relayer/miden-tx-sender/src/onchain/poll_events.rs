use std::collections::HashMap;
use miden_bridge::accounts::token_wrapper::bridge_note_tag;
use miden_client::Client;
use miden_objects::block::BlockNumber;
use miden_objects::utils::ToHex;
use rocket::serde::{Deserialize, Serialize};
use crate::onchain::asset::Asset;
use crate::onchain::errors::OnchainError;
use crate::onchain::OnchainClient;
use crate::utils::felts_to_evm_addresses;
use crate::utils::metadata::decode_slot_into_token_metadata;
use crate::utils::origin::decode_slot_into_origin_info;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct ExitEvent {
    pub note_id: String,
    pub block_number: u32,
    pub asset: Asset,
    pub receiver: String,
    pub destination_chain: u64,
    pub amount: u64,
    pub call_address: Option<String>,
    pub call_data: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct PolledEvents {
    pub chain_tip: u32,
    pub events: Vec<ExitEvent>
}

pub async fn poll_events(
    mut client: &OnchainClient,
    mut storage_client: &mut Client,
    from: BlockNumber
) -> Result<PolledEvents, OnchainError> {
    storage_client.sync_state().await.map_err(OnchainError::from)?;

    let (chain_tip, notes) = client.sync_notes(
        from,
        vec![bridge_note_tag()]
    ).await.map_err(OnchainError::from)?;

    let mut whitelisted_notes = Vec::new();

    let mut tokens = HashMap::new();

    for (note, block) in notes.clone() {
        let account = storage_client.get_account(note.metadata().sender())
            .await.map_err(OnchainError::from)?
            .ok_or(OnchainError::AccountNotFoundInStorage(note.metadata().sender()));
        if let Ok(account) = account {
            tokens.insert(note.metadata().sender().to_hex(), account.account().clone());
            whitelisted_notes.push((note, block))
        }
    }

    Ok(PolledEvents {
        chain_tip,
        events: whitelisted_notes.iter().map(|(event, block_number)| {
            let token_account = tokens.get(&event.metadata().sender().to_hex())
                .unwrap().clone();

            let origin_slot = token_account.storage().slots().get(2).unwrap();
            let metadata_slot = token_account.storage().slots().get(3).unwrap();
            let (origin_network, origin_address) = decode_slot_into_origin_info(
                origin_slot.clone().value()
            ).unwrap();
            let (symbol, decimals) = decode_slot_into_token_metadata(
                metadata_slot.clone().value()
            ).unwrap();

            let receiver_felts = &event.inputs().values()[5..8];
            let receiver_address = felts_to_evm_addresses([
                receiver_felts[2],
                receiver_felts[1],
                receiver_felts[0],
            ]).unwrap();

            ExitEvent {
                note_id: event.id().to_hex(),
                block_number: block_number.as_u32(),
                asset: Asset {
                    origin_address,
                    origin_network,
                    decimals,
                    asset_symbol: symbol.to_str()
                },
                receiver: receiver_address.to_hex_with_prefix(),
                destination_chain: event.inputs().values()[4].as_int(),
                amount: event.inputs().values()[0].as_int(),
                call_data: None,
                call_address: None,
            }
        }).collect()
    })
}