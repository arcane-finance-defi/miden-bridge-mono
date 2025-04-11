extern crate dotenv;
#[macro_use]
extern crate rocket;
mod config;
mod onchain;

use rocket::State as RocketState;
use std::fs::File;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::onchain::client::{client_process_loop, ClientCommand};
use crate::onchain::mint_note::{mint_asset, mint_fungible_asset, Asset, MintArgs, MintedNote};
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
use miden_crypto::hash::rpo::RpoDigest;
use miden_crypto::rand::RpoRandomCoin;
use miden_crypto::Word;
use miden_objects::account::{
    Account, AccountBuilder, AccountId, AccountStorageMode, AccountType, AuthSecretKey,
};
use miden_objects::asset::{FungibleAsset, TokenSymbol};
use miden_objects::block::BlockNumber;
use miden_objects::note::{Note, NoteFile, NoteType};
use miden_objects::utils::ReadAdapter;
use rand::rngs::{StdRng, ThreadRng};
use rand::{rng, Rng, RngCore};
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize, Serialize};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};

/// TODO
fn faucet_id_by_asset(asset: &Asset) -> AccountId {
    AccountId::from_hex("0x774788a75a4698a0000260ad7554a0").unwrap()
}

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
    let recipient = AccountId::from_hex(&mint_args.recipient).map_err(|e| {
        (
            Status::from_code(400).expect("400 is valid status"),
            Json(ErrorResponse { error: e.to_string() }),
        )
    })?;
    let faucet_id = faucet_id_by_asset(&mint_args.asset);

    let (tx, rx) = tokio::sync::oneshot::channel();

    let command = ClientCommand::MintNote {
        faucet_id,
        recipient,
        amount: mint_args.amount,
        asset: mint_args.into_inner().asset,
        tx,
    };

    state.sender.try_send(command).unwrap();

    let mint_result = rx.await.unwrap().unwrap();

    Ok(Json(mint_result))
}

#[get("/chain-tip")]
async fn chain_tip(state: &RocketState<State>) -> String {
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.sender.try_send(ClientCommand::GetChainTip(tx)).unwrap();

    let block_number = rx.await.unwrap();
    block_number.to_string()
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

    let onchain: OnchainClient =
        OnchainClient::new(config.rpc_url().clone(), config.rpc_timeout_ms().clone());

    let (sender, receiver) = tokio::sync::mpsc::channel(10);

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        client_process_loop(onchain, receiver, runtime);
    });

    let onchain: OnchainClient =
        OnchainClient::new(config.rpc_url().clone(), config.rpc_timeout_ms().clone());
    rocket
        .manage(State { client: Arc::new(onchain), sender })
        .mount("/", routes![chain_tip, mint_note])
        .launch()
        .await
        .unwrap();

    Ok(())
}
