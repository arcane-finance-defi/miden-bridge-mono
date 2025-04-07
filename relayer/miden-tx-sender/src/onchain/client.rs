use std::sync::Arc;
use miden_client::block::BlockHeader;
use miden_client::Client;
use miden_client::note::BlockNumber;
use miden_client::rpc::{Endpoint, NodeRpcClient, TonicRpcClient};
use miden_client::store::{StoreAuthenticator};
use miden_client::store::sqlite_store::SqliteStore;
use miden_client::transaction::{LocalTransactionProver, TransactionProver, TransactionRequest};
use miden_crypto::rand::RpoRandomCoin;
use miden_objects::account::{AccountDelta, AccountId, AuthSecretKey};
use miden_objects::Felt;
use rand::Rng;
use crate::onchain::errors::OnchainError;

pub struct OnchainClient {
    rpc: Box<TonicRpcClient>,
    endpoint: Endpoint,
    timeout_ms: u64
}

impl OnchainClient {
    pub fn new(rpc_endpoint: String, timeout_ms: u64) -> Self {
        let endpoint = Endpoint::try_from(rpc_endpoint.as_str()).unwrap();
        OnchainClient {
            rpc: Box::new(TonicRpcClient::new(endpoint.clone(), timeout_ms.clone())),
            endpoint,
            timeout_ms
        }
    }

    pub async fn get_anchor_block(&mut self) -> Result<BlockHeader, OnchainError> {
        let latest_block_height = self.get_chain_tip().await?;

        let epoch = BlockNumber::from(latest_block_height).block_epoch();
        let epoch_block_number = BlockNumber::from_epoch(epoch);

        let (epoch_block_header, _) = self.rpc.get_block_header_by_number(Some(epoch_block_number), false)
            .await.map_err(OnchainError::from)?;

        Ok(epoch_block_header)
    }

    pub async fn get_chain_tip(&mut self) -> Result<BlockNumber, OnchainError> {

        let sync_response = self.rpc.sync_notes(0u32.into(), &[]).await.map_err(OnchainError::from)?;

        let latest_block_height = sync_response.chain_tip;

        Ok(BlockNumber::from(latest_block_height))
    }

    pub async fn execute_tx(
        &mut self,
        tx: TransactionRequest,
        faucet_id: AccountId,
        auth: AuthSecretKey
    ) -> Result<AccountDelta, OnchainError> {

        let mut rng = rand::thread_rng();
        let coin_seed: [u64; 4] = rng.gen();

        let rng = RpoRandomCoin::new(coin_seed.map(Felt::new));


        let tx = {
            let store = SqliteStore::new("./DB.sql".into()).await?;
            let store: Arc<_> = Arc::new(store);

            let rpc = Box::new(TonicRpcClient::new(
                self.endpoint.clone(),
                self.timeout_ms.clone()
            ));

            let mut execution_client = Client::new(
                rpc,
                rng,
                store.clone(),
                Arc::new(StoreAuthenticator::new_with_rng(store.clone(), rng)),
                false
            );

            execution_client.new_transaction(faucet_id, tx).await.map_err(OnchainError::from)?
        };

        let prover = LocalTransactionProver::default();

        let proven_tx = prover.prove(tx.executed_transaction().clone().into())
            .await.map_err(OnchainError::from)?;


        let mut rpc = Box::new(TonicRpcClient::new(
            self.endpoint.clone(),
            self.timeout_ms.clone()
        ));

        rpc.submit_proven_transaction(proven_tx).await.map_err(OnchainError::from)?;

        Ok(tx.executed_transaction().account_delta().clone())
    }
}