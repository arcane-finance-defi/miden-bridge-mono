extern crate dotenv;
#[macro_use]
extern crate rocket;
mod config;
mod onchain;
mod store;
mod utils;

use rocket::State as RocketState;
use std::error::Error;

use crate::config::Config;
use crate::onchain::OnchainClient;
use crate::onchain::client::{ClientCommand, client_process_loop};
use crate::onchain::mint_note::{MintArgs, MintedNote};
use crate::onchain::poll_events::PolledEvents;
use dotenv::dotenv;
use log::warn;
use miden_objects::Word;
use rocket::http::Status;
use rocket::serde::{Deserialize, Serialize, json::Json};
use tokio::sync::mpsc::Sender;

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
    let recipient = Word::parse(&mint_args.recipient)
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
            warn!("{}, source: {}", e, e.source().unwrap());
            Err((Status::InternalServerError, Json(ErrorResponse { error: e.to_string() })))
        },
        Err(e) => {
            warn!("{}, source: {}", e, e.source().unwrap());
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
    state
        .sender
        .try_send(ClientCommand::PollEvents { tx, from_block: from })
        .unwrap();

    match rx.await {
        Ok(Ok(response)) => Ok(Json(response)),
        Ok(Err(_)) | Err(_) => Err(Status::InternalServerError),
    }
}

struct State {
    sender: Sender<ClientCommand>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let rocket = rocket::build();

    let figment = rocket.figment();
    let config: Config = figment.extract().expect("config");

    let mut onchain: OnchainClient =
        OnchainClient::new(config.rpc_url().clone(), config.rpc_timeout_ms());

    let (sender, receiver) = tokio::sync::mpsc::channel(10);

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
        client_process_loop(&mut onchain, receiver, runtime);
    });
    rocket
        .manage(State { sender })
        .mount("/".to_string(), routes![chain_tip, mint_note, poll])
        .launch()
        .await
        .unwrap();

    Ok(())
}
