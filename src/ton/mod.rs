use anyhow::Result;
use tonlib::address::TonAddress;
use tonlib::client::TonClient;
use tonlib::contract::JettonMasterContract;
use tonlib::contract::TonContractFactory;
use tonlib::mnemonic::{KeyPair, Mnemonic};

mod claim;
mod client;
mod transfer;

pub use claim::send_jetton;
pub use client::create_testnet_client;
pub use transfer::transfer_jetton_token;

pub async fn create_key_pair(mnemonic_str: &str) -> anyhow::Result<KeyPair> {
    let mnemonic: Mnemonic = Mnemonic::from_str(mnemonic_str, &None)?;
    let key_pair = mnemonic.to_key_pair()?;
    Ok(key_pair)
}

async fn get_wallet_address(client: &TonClient, owner_address: &str) -> Result<TonAddress> {
    let factory = TonContractFactory::builder(client).build().await?;
    let owner_address = TonAddress::from_base64_url(owner_address)?;
    let contract = factory.get_contract(&owner_address);
    let wallet_address = contract.get_wallet_address(&owner_address).await?;
    Ok(wallet_address)
}
