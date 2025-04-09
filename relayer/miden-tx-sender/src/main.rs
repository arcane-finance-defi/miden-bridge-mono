mod config;
mod onchain;

use std::fs::File;
use rocket::{State as RocketState};
use std::sync::Arc;

#[macro_use] extern crate rocket;
extern crate dotenv;

use dotenv::dotenv;
use miden_client::{Client, ClientError, Felt};
use miden_client::account::component::{BasicFungibleFaucet, BasicWallet, RpoFalcon512};
use miden_client::keystore::FilesystemKeyStore;
use miden_client::rpc::NodeRpcClient;
use miden_client::store::sqlite_store::SqliteStore;
use miden_client::store::{NoteExportType, NoteFilter};
use miden_client::utils::Deserializable;
use miden_client::note::get_input_note_with_id_prefix;
use miden_client::transaction::{TransactionRequest, TransactionRequestBuilder};
use miden_crypto::dsa::rpo_falcon512::SecretKey;
use miden_crypto::rand::RpoRandomCoin;
use miden_crypto::Word;
use miden_objects::account::{Account, AccountBuilder, AccountId, AccountStorageMode, AccountType, AuthSecretKey};
use miden_objects::asset::{FungibleAsset, TokenSymbol};
use miden_objects::block::BlockNumber;
use miden_objects::note::{Note, NoteFile};
use miden_objects::utils::ReadAdapter;
use rand::{rng, Rng, RngCore};
use rand::rngs::{ThreadRng, StdRng};
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

async fn insert_new_fungible_faucet(
    client: &mut Client,
    storage_mode: AccountStorageMode,
    keystore: &FilesystemKeyStore<StdRng>,
) -> Result<(Account, Word), ClientError> {
    let mut rng = rng();

    let key_pair = SecretKey::with_rng(&mut rng);
    let pub_key = key_pair.public_key();

    keystore.add_key(&AuthSecretKey::RpoFalcon512(key_pair)).unwrap();

    // we need to use an initial seed to create the wallet account
    let mut init_seed = [0u8; 32];
    rng.fill_bytes(&mut init_seed);

    let symbol = TokenSymbol::new("TEST").unwrap();
    let max_supply = Felt::try_from(9_999_999_u64.to_le_bytes().as_slice())
        .expect("u64 can be safely converted to a field element");

    let anchor_block = client.get_latest_epoch_block().await.unwrap();

    let (account, seed) = AccountBuilder::new(init_seed)
        .anchor((&anchor_block).try_into().unwrap())
        .account_type(AccountType::FungibleFaucet)
        .storage_mode(storage_mode)
        .with_component(RpoFalcon512::new(pub_key))
        .with_component(BasicFungibleFaucet::new(symbol, 10, max_supply).unwrap())
        .build()
        .unwrap();

    client.add_account(&account, Some(seed), false).await?;
    Ok((account, seed))
}

fn client_process_loop(mut client: OnchainClient, mut receiver: Receiver<ClientCommand>, runtime: Runtime) {
    let store = Arc::new(runtime.block_on(SqliteStore::new("./DB.sql".into())).unwrap());

    let mut rng = rand::thread_rng();
    let coin_seed: [u64; 4] = rng.r#gen();

    let keystore = Arc::new(FilesystemKeyStore::new("./keystore".into()).unwrap());

    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));
    let mut execution_client = Client::new(
        client.rpc.clone(),
        Box::new(rng),
        store.clone(),
        keystore.clone(),
        false
    );

    runtime.block_on(execution_client.sync_state()).unwrap();

    let raw = std::fs::read("./note.mno").unwrap();

    let note_file = NoteFile::read_from_bytes(&raw).unwrap();

    let import = runtime.block_on(execution_client.import_note(note_file)).unwrap();
    println!("imported: {import:?}");

    let account_id = AccountId::from_hex("0xe8e459904bf1548000020aa4a03dc3").unwrap();
    let account = runtime.block_on(execution_client.get_account(account_id)).unwrap();
    println!("Account found: {account:?}");

    // Faucet account generation
    let (faucet, _seed) =
        runtime.block_on(insert_new_fungible_faucet(&mut execution_client, AccountStorageMode::Private, &keystore))
            .unwrap();

    // Test submitting a mint transaction
    let transaction_request = TransactionRequestBuilder::mint_fungible_asset(
        FungibleAsset::new(faucet.id(), 5u64).unwrap(),
        AccountId::from_hex("0x2aa52d28d803dd9000005ee943e671").unwrap(),
        miden_objects::note::NoteType::Private,
        execution_client.rng(),
    )
        .unwrap()
        .build()
        .unwrap();

    let transaction = runtime.block_on(execution_client.new_transaction(faucet.id(), transaction_request)).unwrap();
    runtime.block_on(execution_client.submit_transaction(transaction)).unwrap();

    let mut output_notes = runtime.block_on(execution_client.get_output_notes(NoteFilter::All)).unwrap();
    println!("output: {output_notes:?}");

    let note_file = output_notes.pop().unwrap().into_note_file(&NoteExportType::NoteDetails).unwrap();
    note_file.write("./minted_note.mno").unwrap();

    // println!("Notes {:?}", notes);
    // let note = runtime.block_on(get_input_note_with_id_prefix(&execution_client, notes[0].0.id().to_hex().as_str())).unwrap();
    // println!("Note found: {note:?}, authenticated {}", note.is_authenticated());

    /*
    let transaction_request = TransactionRequestBuilder::new()
        .with_authenticated_input_notes(notes.into_iter().map(|note| (note.0.id(), None)))
        .build()
        .unwrap();

    runtime.block_on(client.execute_tx(transaction_request, account_id)).unwrap();

     */

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
