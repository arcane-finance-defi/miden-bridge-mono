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
fn rocket() -> _ {
    dotenv().ok();

    let rocket = rocket::build();

    let figment = rocket.figment();
    let config: Config = figment.extract().expect("config");

    let onchain: OnchainClient = OnchainClient::new(
        config.rpc_url().clone(),
        config.rpc_timeout_ms().clone()
    );

    rocket.manage(State {client: Arc::new(onchain)}).mount("/", routes![index])
}