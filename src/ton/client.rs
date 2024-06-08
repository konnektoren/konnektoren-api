use anyhow::Result;
use tonlib::client::{ConnectionCheck, TonClient, TonClientBuilder, TonConnectionParams};
//use tonlib::config::TESTNET_CONFIG;
pub const TESTNET_CONFIG: &str = include_str!("./testnet-global.config.json");

pub async fn create_testnet_client() -> Result<TonClient> {
    let params = TonConnectionParams {
        config: TESTNET_CONFIG.to_string(),
        ..Default::default()
    };
    TonClient::set_log_verbosity_level(1);
    let client = TonClientBuilder::new()
        .with_connection_params(&params)
        .with_pool_size(2)
        .with_logging_callback()
        .with_keystore_dir("./var/ton/testnet".to_string())
        //.with_connection_check(ConnectionCheck::Archive)
        .build()
        .await?;
    log::debug!("Client created");
    Ok(client)
}
