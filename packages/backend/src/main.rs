use crate::config::server::server;

mod config;
mod models;
mod services;
mod utility;
mod schema;
mod authenticity;
mod ownership;
mod contract_models;
mod sync;
mod certificate;

#[tokio::main]
async fn main() {
    
    
    server().await.expect("Error!");
}
