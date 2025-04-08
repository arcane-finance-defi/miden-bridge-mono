mod config;
mod onchain;

use std::fs::File;
use rocket::{State as RocketState};
use std::sync::Arc;

#[macro_use] extern crate rocket;
extern crate dotenv;

use dotenv::dotenv;
use miden_client::{Client, Felt};
use miden_client::account::component::{BasicWallet, RpoFalcon512};
use miden_client::rpc::NodeRpcClient;
use miden_client::store::sqlite_store::SqliteStore;
use miden_client::store::{NoteFilter, StoreAuthenticator};
use miden_client::utils::Deserializable;
use miden_crypto::dsa::rpo_falcon512::SecretKey;
use miden_crypto::rand::RpoRandomCoin;
use miden_objects::account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType, AuthSecretKey};
use miden_objects::block::BlockNumber;
use miden_objects::note::{Note, NoteFile};
use miden_objects::utils::ReadAdapter;
use rand::Rng;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot::Sender as OneshotSender;
use crate::config::Config;
use crate::onchain::OnchainClient;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/chain-tip")]
async fn chain_tip(state: &RocketState<State>) -> String {
    // let tip = state.client.get_chain_tip().await.unwrap();
    // println!("Miden sync chain tip {tip}");
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.sender.try_send(ClientCommand::GetChainTip(tx)).unwrap();

    let block_number = rx.await.unwrap();
    println!("chain-tip received {block_number}");

    block_number.to_string()
}

struct State {
    client: Arc<OnchainClient>,
    sender: Sender<ClientCommand>,
}

pub enum ClientCommand {
    GetChainTip(OneshotSender<BlockNumber>),
}

fn client_process_loop(mut client: OnchainClient, mut receiver: Receiver<ClientCommand>, runtime: Runtime) {
    let store = Arc::new(runtime.block_on(SqliteStore::new("./DB.sql".into())).unwrap());

    let mut rng = rand::thread_rng();
    let coin_seed: [u64; 4] = rng.r#gen();

    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));
    let mut execution_client = Client::new(
        client.rpc,
        Box::new(rng),
        store.clone(),
        Arc::new(StoreAuthenticator::new_with_rng(store.clone(), rng)),
        false
    );

    runtime.block_on(execution_client.sync_state()).unwrap();

    let account_id = AccountId::from_hex("0x36f843ab937b548000014059693385").unwrap();
    let account = runtime.block_on(execution_client.get_account(account_id)).unwrap().unwrap();
    println!("Account {:?}", account.account());

    // let note_file = NoteFile::
    // execution_client.import_note();

    let mut note_file = File::open("./note.mno").unwrap();
    let mut read_adapter = ReadAdapter::new(&mut note_file);
    let note = Note::read_from(&mut read_adapter).unwrap();

    println!("Note {note:?}");

    let notes = runtime.block_on(execution_client.get_input_notes(NoteFilter::All)).unwrap();
    println!("Notes {:?}", notes);

    loop {
        let command = runtime.block_on(receiver.recv()).unwrap();

        match command {
            ClientCommand::GetChainTip(sender) => {
                let tip = runtime.block_on(execution_client.get_sync_height()).unwrap();
                sender.send(tip).unwrap();
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let rocket = rocket::build();

    let figment = rocket.figment();
    let config: Config = figment.extract().expect("config");

    let onchain: OnchainClient = OnchainClient::new(
        config.rpc_url().clone(),
        config.rpc_timeout_ms().clone()
    );

    let (sender, receiver) = tokio::sync::mpsc::channel(10);

    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        client_process_loop(onchain, receiver, runtime);
    });

    let onchain: OnchainClient = OnchainClient::new(
        config.rpc_url().clone(),
        config.rpc_timeout_ms().clone()
    );
    rocket.manage(State {client: Arc::new(onchain), sender}).mount("/", routes![index, chain_tip]).launch().await.unwrap();

    Ok(())
}
