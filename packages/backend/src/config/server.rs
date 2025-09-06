use crate::authenticity::authenticity_event_listener::listen_for_authenticity_events;
use crate::config::app_router::RouterPath;
use crate::config::app_router::paths;
use crate::config::app_state::AppState;
use crate::ownership::ownership_event::listen_for_ownership_events;
use anyhow::{Result, anyhow};
use axum::Router;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::{Duration, sleep};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn server() -> Result<()> {
    eprintln!("PROJECT STARTING...");
    // Load environment variables
    dotenv().ok();

    let arc_state = Arc::from(AppState::init_app_state().await.unwrap());

    // Run migrations
    let mut attempts = 3;
    let mut conn = None;
    for _ in 0..attempts {
        match arc_state.db_pool.get() {
            Ok(c) => {
                conn = Some(c);
                break;
            }
            Err(e) => {
                eprintln!(
                    "Failed to get DB connection (attempt {}): {:?}",
                    4 - attempts,
                    e
                );
                attempts -= 1;
                if attempts == 0 {
                    return Err(anyhow!("Failed to get DB connection after retries: {}", e));
                }
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
    let mut conn = conn.unwrap();

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| {
            eprintln!("Failed to run migrations: {:?}", e);
            eyre::eyre!("Migration failed: {}", e)
        })
        .unwrap();
    eprintln!("Database migrations completed successfully");

    let state_clone1 = arc_state.clone();
    let state_clone2 = arc_state.clone();

    tokio::spawn(async move {
        let mut attempts = 5;
        loop {
            if let Err(e) = listen_for_authenticity_events(&state_clone1).await {
                eprintln!(
                    "Error in authenticity listener (attempt {}): {:?}",
                    6 - attempts,
                    e
                );
                attempts -= 1;
                if attempts == 0 {
                    eprintln!("Max retries reached for authenticity listener");
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            } else {
                break;
            }
        }
    });

    tokio::spawn(async move {
        let mut attempts = 5;
        loop {
            if let Err(e) = listen_for_ownership_events(&state_clone2).await {
                eprintln!(
                    "Error in ownership listener (attempt {}): {:?}",
                    6 - attempts,
                    e
                );
                attempts -= 1;
                if attempts == 0 {
                    eprintln!("Max retries reached for ownership listener");
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            } else {
                break;
            }
        }
    });

    // let mut conn = arc_state
    //     .db_pool
    //     .get()
    //     .map_err(|e| eyre::eyre!("Failed to get DB connection: {}", e)).unwrap();
    // conn.run_pending_migrations(MIGRATIONS)
    //     .map_err(|e| eyre::eyre!("Migration failed: {}", e)).unwrap();
    //
    //
    // let state_clone1 = arc_state.clone();
    // let state_clone2 = arc_state.clone();
    //
    // tokio::spawn(async move {
    //     if let Err(e) = listen_for_authenticity_events(&state_clone1).await {
    //         eprintln!("Error in authenticity listener, retrying in 5s: {:?}", e);
    //         tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    //     }
    // });
    //
    // tokio::spawn(async move {
    //     if let Err(e) = listen_for_ownership_events(&state_clone2).await {
    //         eprintln!("Error in event listener for ownership: {:?}", e);
    //         tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    //     }
    // });

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
