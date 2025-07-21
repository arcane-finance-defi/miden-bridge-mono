extern crate dotenv;
#[macro_use]
extern crate rocket;
mod config;
mod onchain;
mod store;
mod utils;

use std::error::Error;
use rocket::State as RocketState;
use std::sync::Arc;

use crate::config::Config;
use crate::onchain::client::{client_process_loop, ClientCommand};
use crate::onchain::mint_note::{mint_asset, MintArgs, MintedNote};
use crate::onchain::OnchainClient;
use dotenv::dotenv;
use miden_bridge::accounts::token_wrapper::TokenWrapperAccount;
use miden_client::account::component::{BasicFungibleFaucet, BasicWallet, RpoFalcon512};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::get_input_note_with_id_prefix;
use miden_client::rpc::NodeRpcClient;
use miden_client::store::sqlite_store::SqliteStore;
use miden_client::store::{NoteExportType, NoteFilter};
use miden_client::transaction::{TransactionRequest, TransactionRequestBuilder};
use miden_client::utils::Deserializable;
use miden_client::{Client, ClientError, Felt};
use miden_crypto::dsa::rpo_falcon512::SecretKey;
use miden_objects::crypto::hash::rpo::RpoDigest;
use miden_crypto::rand::RpoRandomCoin;
use miden_crypto::Word;
use miden_objects::account::{
    Account, AccountBuilder, AccountId, AccountStorageMode, AccountType, AuthSecretKey,
};
use miden_objects::asset::{FungibleAsset, TokenSymbol};
use miden_objects::block::BlockNumber;
use miden_objects::note::{Note, NoteFile, NoteType};
use miden_objects::utils::{ReadAdapter, parse_hex_string_as_word};
use rand::rngs::{StdRng, ThreadRng};
use rand::{rng, Rng, RngCore};
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};
use log::warn;
use crate::onchain::poll_events::PolledEvents;

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ErrorResponse {
    error: String,
}

#[post("/mint", format = "json", data = "<mint_args>")]
async fn mint_note(
    mint_args: Json<MintArgs>,
    state: &RocketState<State>,
) -> Result<Json<MintedNote>, (Status, Json<ErrorResponse>)> {
    let recipient = parse_hex_string_as_word(&mint_args.recipient)
        .map_err(|e| (Status::BadRequest, Json(ErrorResponse { error: e.to_string() })))?;
    let (tx, rx) = tokio::sync::oneshot::channel();

    let command = ClientCommand::MintNote {
        recipient,
        amount: mint_args.amount,
        asset: mint_args.into_inner().asset,
        tx,
    };

    if let Err(e) = state.sender.try_send(command) {
        return Err((Status::InternalServerError, Json(ErrorResponse { error: e.to_string() })));
    }

    match rx.await {
        Ok(Ok(mint_result)) => Ok(Json(mint_result)),
        Ok(Err(e)) => {
            warn!("{}, source: {}", e, e.source().unwrap().to_string());
            Err((Status::InternalServerError, Json(ErrorResponse { error: e.to_string() })))
        },
        Err(e) => {
            warn!("{}, source: {}", e, e.source().unwrap().to_string());
            Err((Status::InternalServerError, Json(ErrorResponse { error: e.to_string() })))
        },
    }
}

#[get("/chain-tip")]
async fn chain_tip(state: &RocketState<State>) -> Result<String, Status> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.sender.try_send(ClientCommand::GetChainTip(tx)).unwrap();

    match rx.await {
        Ok(Ok(block_number)) => Ok(block_number.to_string()),
        Ok(Err(_)) | Err(_) => Err(Status::InternalServerError),
    }
}

#[get("/poll?<from>")]
async fn poll(from: u32, state: &RocketState<State>) -> Result<Json<PolledEvents>, Status> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.sender.try_send(ClientCommand::PollEvents{ tx, from_block: from }).unwrap();

    match rx.await {
        Ok(Ok(response)) => Ok(Json(response)),
        Ok(Err(_)) | Err(_) => Err(Status::InternalServerError),
    }
}

struct State {
    client: Arc<OnchainClient>,
    sender: Sender<ClientCommand>,
}

#[derive(Debug)]
enum MintNoteError {}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let rocket = rocket::build();

    let figment = rocket.figment();
    let config: Config = figment.extract().expect("config");

    let mut onchain: OnchainClient =
        OnchainClient::new(config.rpc_url().clone(), config.rpc_timeout_ms().clone());

    let (sender, receiver) = tokio::sync::mpsc::channel(10);

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        client_process_loop(&mut onchain, receiver, runtime);
    });

    let onchain: OnchainClient =
        OnchainClient::new(config.rpc_url().clone(), config.rpc_timeout_ms().clone());
    rocket
        .manage(State { client: Arc::new(onchain), sender })
        .mount("/".to_string(), routes![chain_tip, mint_note, poll])
        .launch()
        .await
        .unwrap();

    Ok(())
}
