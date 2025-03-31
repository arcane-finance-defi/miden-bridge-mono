use std::collections::BTreeMap;
use miden_client::store::{AccountRecord, AccountStatus, ChainMmrNodeFilter, InputNoteRecord, NoteFilter, OutputNoteRecord, Store, StoreError, TransactionFilter};
use miden_client::sync::{NoteTagRecord, StateSyncUpdate};
use miden_client::transaction::{TransactionRecord, TransactionStoreUpdate};
use miden_crypto::hash::rpo::RpoDigest as Digest;
use miden_crypto::merkle::{InOrderIndex, MmrPeaks};
use miden_crypto::utils::word_to_hex;
use miden_crypto::Word;
use miden_objects::account::{Account, AccountCode, AccountHeader, AccountId, AuthSecretKey};
use miden_objects::block::{BlockHeader, BlockNumber};
use crate::onchain::client::OnchainClient;

pub struct MockStore<'a> {
    client: &'a mut OnchainClient,
    provided_auth: AuthSecretKey
}

impl<'a> MockStore<'a> {
    pub fn new(rpc: &'a mut OnchainClient, provided_auth: AuthSecretKey) -> MockStore<'a> {
        MockStore {
            client: rpc,
            provided_auth
        }
    }
}

impl Store for MockStore<'_> {

    fn get_current_timestamp(&self) -> Option<u64> {
        Some(0u64)
    }

    async fn get_transactions(&self, _filter: TransactionFilter) -> Result<Vec<TransactionRecord>, StoreError> {
        todo!()
    }

    async fn apply_transaction(&self, _tx_update: TransactionStoreUpdate) -> Result<(), StoreError> {
        todo!()
    }

    async fn get_input_notes(&self, _filter: NoteFilter) -> Result<Vec<InputNoteRecord>, StoreError> {
        Ok(vec![])
    }

    async fn get_output_notes(&self, _filter: NoteFilter) -> Result<Vec<OutputNoteRecord>, StoreError> {
        todo!()
    }

    async fn upsert_input_notes(&self, _notes: &[InputNoteRecord]) -> Result<(), StoreError> {
        Ok(())
    }

    async fn get_block_headers(&self, _block_numbers: &[BlockNumber]) -> Result<Vec<(BlockHeader, bool)>, StoreError> {
        todo!()
    }

    async fn get_tracked_block_headers(&self) -> Result<Vec<BlockHeader>, StoreError> {
        todo!()
    }

    async fn get_chain_mmr_nodes(&self, _filter: ChainMmrNodeFilter) -> Result<BTreeMap<InOrderIndex, Digest>, StoreError> {
        todo!()
    }

    async fn insert_chain_mmr_nodes(&self, _nodes: &[(InOrderIndex, Digest)]) -> Result<(), StoreError> {
        todo!()
    }

    async fn get_chain_mmr_peaks_by_block_num(&self, _block_num: BlockNumber) -> Result<MmrPeaks, StoreError> {
        todo!()
    }

    async fn insert_block_header(&self, _block_header: BlockHeader, _chain_mmr_peaks: MmrPeaks, _has_client_notes: bool) -> Result<(), StoreError> {
        todo!()
    }

    async fn get_account_ids(&self) -> Result<Vec<AccountId>, StoreError> {
        todo!()
    }

    async fn get_account_headers(&self) -> Result<Vec<(AccountHeader, AccountStatus)>, StoreError> {
        todo!()
    }

    async fn get_account_header(&self, _account_id: AccountId) -> Result<Option<(AccountHeader, AccountStatus)>, StoreError> {
        todo!()
    }

    async fn get_account_header_by_hash(&self, _account_hash: Digest) -> Result<Option<AccountHeader>, StoreError> {
        todo!()
    }

    async fn get_account(&self, _account_id: AccountId) -> Result<Option<AccountRecord>, StoreError> {
        todo!()
    }

    async fn get_account_auth_by_pub_key(&self, pub_key: Word) -> Result<Option<AuthSecretKey>, StoreError> {
        let provided_pubkey: Word = match self.provided_auth.clone() {
            AuthSecretKey::RpoFalcon512(secret) => secret.public_key()
        }.into();

        if pub_key == provided_pubkey {
            Ok(Some(self.provided_auth.clone()))
        } else {
            Err(StoreError::AccountKeyNotFound(word_to_hex(&pub_key)
                .map_err(|_| StoreError::DatabaseError("Invalid provided auth".into()))?
            ))
        }
    }

    async fn get_account_auth(&self, _account_id: AccountId) -> Result<Option<AuthSecretKey>, StoreError> {
        todo!()
    }

    async fn insert_account(&self, _account: &Account, _account_seed: Option<Word>, _auth_info: &AuthSecretKey) -> Result<(), StoreError> {
        todo!()
    }

    async fn upsert_foreign_account_code(&self, _account_id: AccountId, _code: AccountCode) -> Result<(), StoreError> {
        todo!()
    }

    async fn get_foreign_account_code(&self, _account_ids: Vec<AccountId>) -> Result<BTreeMap<AccountId, AccountCode>, StoreError> {
        todo!()
    }

    async fn update_account(&self, _new_account_state: &Account) -> Result<(), StoreError> {
        todo!()
    }

    async fn get_note_tags(&self) -> Result<Vec<NoteTagRecord>, StoreError> {
        todo!()
    }

    async fn add_note_tag(&self, _tag: NoteTagRecord) -> Result<bool, StoreError> {
        todo!()
    }

    async fn remove_note_tag(&self, _tag: NoteTagRecord) -> Result<usize, StoreError> {
        todo!()
    }

    async fn get_sync_height(&mut self) -> Result<BlockNumber, StoreError> {
        let result = self.client.get_chain_tip().await.map_err(|e| StoreError::DatabaseError(e.to_string()));
        result
    }

    async fn apply_state_sync(&self, _state_sync_update: StateSyncUpdate) -> Result<(), StoreError> {
        todo!()
    }
}