use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct Asset {
    pub origin_network: u32,
    pub origin_address: String,
    pub asset_symbol: String,
    pub decimals: u8,
}