use crate::config::app_router::RouterPath;
use crate::config::app_router::paths;
use crate::config::app_state::AppState;
use crate::authenticity::authenticity_event_listener::listen_for_authenticity_events;
use crate::ownership::ownership_event::listen_for_ownership_events;
use anyhow::Result;
use axum::Router;
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn server() -> Result<()> {
    eprintln!("PROJECT STARTING...");
    // Load environment variables
    dotenv().ok();

    let arc_state = Arc::from(AppState::init_app_state().await.unwrap());

    let state_clone1 = arc_state.clone();
    let state_clone2 = arc_state.clone();

    tokio::spawn(async move {
            if let Err(e) = listen_for_authenticity_events(&state_clone1).await {
                eprintln!("Error in authenticity listener, retrying in 5s: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
    });

    tokio::spawn(async move {
            if let Err(e) = listen_for_ownership_events(&state_clone2).await {
                eprintln!("Error in event listener for ownership: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
    });

    // Define routes
    let app: Router = paths(arc_state, RouterPath::init());

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = TcpListener::bind(addr).await?;

    eprintln!("Server running on {:?}", addr);
    eprintln!("Swagger UI available at {:?}/swagger-ui/index.html#/", addr);

    axum::serve(listener, app).await?;

    Ok(()) // another way to say return nothing
}

//TODO: https://eri-eth-ui.vercel.app/verify?cert=%7B%22name%22%3A%22Jaguar+A15%22%2C%22uniqueId%22%3A%22JAG15%22%2C%22serial%22%3A%22122121%22%2C%22date%22%3A1755909120%2C%22owner%22%3A%220xF2E7E2f51D7C9eEa9B0313C2eCa12f8e43bd1855%22%2C%22metadataHash%22%3A%220xa11af94997a5c7478c4d105198747f3dc308f60d7237b99b49f58a215a52f059%22%2C%22metadata%22%3A%5B%22GREY%22%2C%22DOUBLE+EXHAUST%22%5D%7D&sig=0xad71ff20241e4a798f4416f71411b4343ed8f939c45375d4f78125a84fdfd1fd0ca5f8d0b037cddb02bf2989b2c1b0a9e0845260867160172e07cb0cc452148b1c
