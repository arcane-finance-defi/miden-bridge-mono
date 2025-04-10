use crate::MintNoteError;
use crate::onchain::deploy_token::insert_new_fungible_faucet;
use crate::onchain::errors::OnchainError;
use crate::onchain::mint_note::{Asset, MintedNote, mint_fungible_asset};
use miden_client::Client;
use miden_client::block::BlockHeader;
use miden_client::keystore::FilesystemKeyStore;
use miden_client::note::BlockNumber;
use miden_client::rpc::{Endpoint, NodeRpcClient, TonicRpcClient};
use miden_client::store::NoteExportType;
use miden_client::store::sqlite_store::SqliteStore;
use miden_client::transaction::{
    LocalTransactionProver, TransactionProver, TransactionRequest, TransactionResult,
};
use miden_crypto::rand::RpoRandomCoin;
use miden_objects::Felt;
use miden_objects::account::{AccountDelta, AccountId, AccountStorageMode};
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
    let tx = execution_client
        .new_transaction(faucet_id, tx)
        .await
        .map_err(OnchainError::from)?;

    execution_client
        .submit_transaction(tx.clone())
        .await
        .map_err(OnchainError::from)?;

    Ok(tx)
}

pub enum ClientCommand {
    GetChainTip(OneshotSender<BlockNumber>),
    MintNote {
        faucet_id: AccountId,
        recipient: AccountId,
        amount: u64,
        asset: Asset,
        tx: OneshotSender<Result<MintedNote, MintNoteError>>,
    },
}

pub fn client_process_loop(
    mut client: OnchainClient,
    mut receiver: Receiver<ClientCommand>,
    runtime: Runtime,
) {
    let store = Arc::new(runtime.block_on(SqliteStore::new("./DB.sql".into())).unwrap());

    let mut rng = rand::rng();
    let coin_seed: [u64; 4] = rng.random();

    let keystore = Arc::new(FilesystemKeyStore::new("./keystore".into()).unwrap());

    let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));
    let mut execution_client =
        Client::new(client.rpc.clone(), Box::new(rng), store.clone(), keystore.clone(), false);

    runtime.block_on(execution_client.sync_state()).unwrap();

    loop {
        let command = runtime.block_on(receiver.recv()).unwrap();

        match command {
            ClientCommand::GetChainTip(sender) => {
                let tip = runtime.block_on(execution_client.get_sync_height()).unwrap();
                sender.send(tip).unwrap();
            },
            ClientCommand::MintNote { faucet_id, recipient, amount, asset, tx } => {
                let now = Instant::now();
                runtime.block_on(execution_client.sync_state()).unwrap();

                let faucet_account =
                    runtime.block_on(execution_client.get_account(faucet_id)).unwrap();
                if faucet_account.is_none() {
                    runtime
                        .block_on(insert_new_fungible_faucet(
                            &mut execution_client,
                            AccountStorageMode::Private,
                            &keystore,
                            &asset.asset_symbol,
                            asset.decimals,
                        ))
                        .unwrap();
                }

                let mint_result = runtime
                    .block_on(mint_fungible_asset(
                        &mut execution_client,
                        faucet_id,
                        recipient,
                        amount,
                    ))
                    .unwrap();
                let note_id = mint_result.created_notes().get_note(0).id();
                let output_note =
                    runtime.block_on(execution_client.get_output_note(note_id)).unwrap().unwrap();

                let timestamp =
                    SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");

                let note_file = output_note.into_note_file(&NoteExportType::NoteDetails).unwrap();
                let path = format!("./minted_note_wrapper_{}.mno", timestamp.as_secs());
                note_file.write(path).unwrap();

                println!("Minting took {}", now.elapsed().as_millis());

                tx.send(Ok(MintedNote {
                    note_id: note_id.to_hex(),
                    faucet_id: faucet_id.to_hex(),
                    transaction_id: mint_result.executed_transaction().id().to_hex(),
                }))
                .unwrap();
            },
        }
    }
}
