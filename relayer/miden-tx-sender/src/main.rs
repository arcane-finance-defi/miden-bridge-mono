mod config;
mod onchain;

use std::sync::Arc;

#[macro_use] extern crate rocket;
extern crate dotenv;

use dotenv::dotenv;
use crate::config::Config;
use crate::onchain::OnchainClient;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

struct State {
    client: Arc<OnchainClient>
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let rocket = rocket::build();

    let figment = rocket.figment();
    let config: Config = figment.extract().expect("config");

    let mut onchain: OnchainClient = OnchainClient::new(
        config.rpc_url().clone(),
        config.rpc_timeout_ms().clone()
    );

    let block = onchain.get_anchor_block().await.unwrap().block_num();
    println!("{block}");
    rocket.manage(State {client: Arc::new(onchain)}).mount("/", routes![index])
}