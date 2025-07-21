use async_sqlite::{Pool, JournalMode, PoolBuilder};
use miden_client::store::StoreError;
use miden_client::utils::{Deserializable, Serializable};
use miden_objects::account::AccountId;
use rusqlite::params;
use std::path::PathBuf;

pub struct Store {
    pool: Pool,
}

impl Store {
    pub async fn new(database_filepath: PathBuf) -> Result<Self, StoreError> {
        let database_exists = database_filepath.exists();

        let pool = PoolBuilder::new()
            .path(database_filepath)
            .journal_mode(JournalMode::Wal)
            .open()
            .await
            .map_err(|err| StoreError::DatabaseError(err.to_string()))?;

        if !database_exists {
            pool.conn_mut(|conn| conn.execute_batch(include_str!("store.sql")))
                .await
                .map_err(|err| StoreError::DatabaseError(err.to_string()))?;
        }

        Ok(Self { pool })
    }

    pub async fn get_faucet_id(
        &self,
        origin_network: u32,
        origin_address: &str,
    ) -> Result<Option<AccountId>, StoreError> {
        let origin_address = origin_address.to_string(); // move into closure

        let result = self.pool
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT miden_faucet_id FROM assets_info WHERE origin_network = ?1 AND origin_address = ?2"
                )?;
                match stmt.query_row(params![origin_network, origin_address], |row| {
                    let blob: Vec<u8> = row.get(0)?;
                    let id = AccountId::read_from_bytes(&blob)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            blob.len(), rusqlite::types::Type::Blob, Box::new(e)
                        ))?;
                    Ok(id)
                }) {
                    Ok(id) => Ok(Some(id)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(e),
                }
            })
            .await
            .map_err(|err| StoreError::DatabaseError(err.to_string()))?;

        Ok(result)
    }

    pub async fn add_faucet_id(
        &self,
        origin_network: u32,
        origin_address: &str,
        faucet_id: &AccountId,
    ) -> Result<(), StoreError> {
        let origin_address = origin_address.to_string();
        let faucet_id_bytes = faucet_id.to_bytes();

        self.pool
            .conn_mut(move |conn| {
                conn.execute(
                    "INSERT INTO assets_info (origin_network, origin_address, miden_faucet_id)
                     VALUES (?1, ?2, ?3)",
                    params![origin_network, origin_address, faucet_id_bytes],
                )
                .map(|_| ())
            })
            .await
            .map_err(|err| StoreError::DatabaseError(err.to_string()))?;

        Ok(())
    }
}
