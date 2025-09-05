use crate::authenticity::authenticity_abi::TrueAuthenticity;
use crate::ownership::ownership_abi::TrueOwnership;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use ecdsa::SigningKey;
use ethabi::ethereum_types::Address;
use ethers::core::k256::Secp256k1;
use ethers::middleware::{Middleware, SignerMiddleware};
use ethers::prelude::{Http, LocalWallet, Provider};
use ethers::signers::{Signer, Wallet};
use eyre::Report;
use std::env;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub authenticity_contract: TrueAuthenticity<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>,
    pub ownership_contract: TrueOwnership<SignerMiddleware<Provider<Http>, Wallet<SigningKey<Secp256k1>>>>,
}

impl AppState {
    pub async fn init_app_state() -> anyhow::Result<AppState, Report> {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| eyre::eyre!("Failed to create pool: {}", e))?;

        //contract connection
        let rpc_url = env::var("BASE_URL")?;
        let private_key = env::var("PRIVATE_KEY")?;
        let authenticity_address: Address = env::var("AUTHENTICITY_ADDRESS")?
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid contract address"))
            .unwrap();
        let ownership_address: Address = env::var("OWNERSHIP_ADDRESS")?
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid contract address"))
            .unwrap();

        let provider = Provider::<Http>::try_from(&rpc_url)?.interval(Duration::from_millis(1000));
        let chain_id = provider.get_chainid().await?.as_u64();

        let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        println!("Wallet address: 0x{:x}", wallet.address());

        let eth_client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

        let authenticity_contract = TrueAuthenticity::new(authenticity_address, eth_client.clone());
        let ownership_contract = TrueOwnership::new(ownership_address, eth_client.clone());

        let state = AppState {
            db_pool: pool,
            authenticity_contract,
            ownership_contract,
        };
        
        Ok(state)
    }
}
