use crate::onchain::deploy_token::insert_new_fungible_faucet;
use crate::onchain::errors::OnchainError;
use crate::onchain::mint_note::{mint_asset, Asset, MintedNote};
use crate::store::Store;
use miden_client::block::BlockHeader;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::BlockNumber;
use miden_client::rpc::{Endpoint, NodeRpcClient, TonicRpcClient};
use miden_client::store::sqlite_store::SqliteStore;
use miden_client::store::NoteExportType;
use miden_client::transaction::{
    LocalTransactionProver, TransactionProver, TransactionRequest, TransactionResult,
};
use miden_client::Client;
use miden_crypto::rand::RpoRandomCoin;
use miden_objects::account::{AccountDelta, AccountId, AccountStorageMode};
use miden_objects::{Felt, Word};
use rand::rngs::StdRng;
use rand::Rng;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender as OneshotSender;

pub struct OnchainClient {
    pub rpc: Arc<dyn NodeRpcClient + Send + Sync + 'static>,
    endpoint: Endpoint,
    timeout_ms: u64,
}

impl OnchainClient {
    pub fn new(rpc_endpoint: String, timeout_ms: u64) -> Self {
        let endpoint = Endpoint::try_from(rpc_endpoint.as_str()).unwrap();
        OnchainClient {
            rpc: Arc::new(TonicRpcClient::new(&endpoint, timeout_ms.clone())),
            endpoint,
            timeout_ms,
        }
    }

    pub async fn get_anchor_block(&mut self) -> Result<BlockHeader, OnchainError> {
        let latest_block_height = self.get_chain_tip().await?;

        let epoch = BlockNumber::from(latest_block_height).block_epoch();
        let epoch_block_number = BlockNumber::from_epoch(epoch);

        let (epoch_block_header, _) = self
            .rpc
            .get_block_header_by_number(Some(epoch_block_number), false)
            .await
            .map_err(OnchainError::from)?;

        Ok(epoch_block_header)
    }

    pub async fn get_chain_tip(&mut self) -> Result<BlockNumber, OnchainError> {
        let sync_response =
            self.rpc.sync_notes(0u32.into(), &[]).await.map_err(OnchainError::from)?;

        let latest_block_height = sync_response.chain_tip;

        Ok(BlockNumber::from(latest_block_height))
    }

    pub async fn execute_tx(
        &mut self,
        tx: TransactionRequest,
        faucet_id: AccountId,
    ) -> Result<AccountDelta, OnchainError> {
        let mut rng = rand::thread_rng();
        let coin_seed: [u64; 4] = rng.r#gen();

        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));

        let tx = {
            let store = SqliteStore::new("./DB.sql".into()).await?;
            let store: Arc<_> = Arc::new(store);

            let rpc = Arc::new(TonicRpcClient::new(&self.endpoint, self.timeout_ms.clone()));

            let mut execution_client =
                Client::new(rpc, Box::new(rng), store.clone(), Arc::new(()), false);

            execution_client
                .new_transaction(faucet_id, tx)
                .await
                .map_err(OnchainError::from)?
        };

        let prover = LocalTransactionProver::default();

        let proven_tx = prover
            .prove(tx.executed_transaction().clone().into())
            .await
            .map_err(OnchainError::from)?;

        let mut rpc = Box::new(TonicRpcClient::new(&self.endpoint, self.timeout_ms.clone()));

        rpc.submit_proven_transaction(proven_tx).await.map_err(OnchainError::from)?;

        Ok(tx.executed_transaction().account_delta().clone())
    }
}

pub async fn execute_tx(
    execution_client: &mut Client,
    tx: TransactionRequest,
    faucet_id: AccountId,
) -> Result<TransactionResult, OnchainError> {
    let tx = execution_client.new_transaction(faucet_id, tx).await?;

    execution_client.submit_transaction(tx.clone()).await?;

    Ok(tx)
}

pub enum ClientCommand {
    GetChainTip(OneshotSender<Result<BlockNumber, OnchainError>>),
    MintNote {
        recipient: Word,
        amount: u64,
        asset: Asset,
        tx: OneshotSender<Result<MintedNote, OnchainError>>,
    },
}

async fn get_sync_height(execution_client: &mut Client) -> Result<BlockNumber, OnchainError> {
    execution_client.sync_state().await?;
    execution_client.get_sync_height().await.map_err(OnchainError::from)
}

async fn mint_note(
    execution_client: &mut Client,
    keystore: &FilesystemKeyStore<StdRng>,
    assets_store: &Store,
    recipient: Word,
    amount: u64,
    asset: Asset,
) -> Result<MintedNote, OnchainError> {
    let now = Instant::now();
    execution_client.sync_state().await?;

    let faucet_id =
        match assets_store.get_faucet_id(asset.origin_network, &asset.origin_address).await? {
            Some(id) => id,
            None => {
                let (account, _) = insert_new_fungible_faucet(
                    execution_client,
                    AccountStorageMode::Private,
                    &keystore,
                    &asset.asset_symbol,
                    asset.decimals,
                )
                .await?;

                let account_id = account.id();
                assets_store
                    .add_faucet_id(asset.origin_network, &asset.origin_address, &account_id)
                    .await?;

                account_id
            },
        };

    let mint_result = mint_asset(execution_client, faucet_id, recipient, amount).await?;
    let note_id = mint_result.created_notes().get_note(0).id();

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");

    println!("Minting took {}", now.elapsed().as_millis());

    Ok(MintedNote {
        note_id: note_id.to_hex(),
        faucet_id: faucet_id.to_hex(),
        transaction_id: mint_result.executed_transaction().id().to_hex(),
    })
}

pub fn client_process_loop(
    mut client: OnchainClient,
    mut receiver: Receiver<ClientCommand>,
    runtime: Runtime,
) {
    let miden_client_store =
        Arc::new(runtime.block_on(SqliteStore::new("./miden_store.sql".into())).unwrap());
    let assets_store = runtime
        .block_on(Store::new("./assets_store.sql".into()))
        .expect("Assets store to be initialized");

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();

    let keystore = Arc::new(FilesystemKeyStore::new("./keystore".into()).unwrap());

    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));
    let mut execution_client =
        Client::new(client.rpc.clone(), Box::new(rng), miden_client_store, keystore.clone(), false);

    runtime.block_on(execution_client.sync_state()).unwrap();

    loop {
        let command = runtime.block_on(receiver.recv()).unwrap();

        match command {
            ClientCommand::GetChainTip(sender) => {
                let result = runtime.block_on(get_sync_height(&mut execution_client));
                sender.send(result).unwrap();
            },
            ClientCommand::MintNote { recipient, amount, asset, tx } => {
                let result = runtime.block_on(mint_note(
                    &mut execution_client,
                    &keystore,
                    &assets_store,
                    recipient,
                    amount,
                    asset,
                ));

                tx.send(result).unwrap();
            },
        }
    }
}
