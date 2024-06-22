use anyhow::Result;
use base64::prelude::*;
use num_bigint::BigUint;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tonlib::address::TonAddress;
use tonlib::cell::{ArcCell, BagOfCells, Cell, CellBuilder};
use tonlib::client::{TonClient, TonClientInterface};
use tonlib::contract::{JettonMasterContract, TonContractFactory};
use tonlib::message::{JettonTransferMessage, TransferMessage};
use tonlib::mnemonic::{KeyPair, Mnemonic};
use tonlib::wallet::{TonWallet, WalletVersion};

pub async fn create_transfer_cell(
    client: &TonClient,
    key_pair: &KeyPair,
    contract_address: &str,
    recipient_address: &str,
    amount: u128,
) -> Result<(Cell, TonAddress)> {
    let wallet = TonWallet::derive_default(WalletVersion::V4R2, &key_pair)?;

    let user_address = wallet.address;

    let contract_address = TonAddress::from_base64_url(contract_address)?;
    let recipient_address = TonAddress::from_base64_url(recipient_address)?;

    let factory = TonContractFactory::builder(&client).build().await?;

    let jetton_contract = factory.get_contract(&contract_address);

    let user_jetton_wallet = jetton_contract.get_wallet_address(&user_address).await?;

    let mut writer = CellBuilder::new();
    let forward_payload: ArcCell = writer
        .store_uint(32, &0u32.into())?
        .store_string("Claim")?
        .build()?
        .to_arc();

    let transfer_cell: Cell = writer
        .store_uint(32, &0xf8a7ea5u32.into())?
        .store_uint(64, &0u32.into())?
        .store_coins(&(amount * 1_000_000_000).into())?
        .store_address(&recipient_address.into())?
        .store_address(&user_address.into())?
        .store_bit(false)?
        .store_coins(&1u32.into())?
        .store_bit(true)?
        .store_reference(&forward_payload)?
        .build()?;

    Ok((transfer_cell, user_jetton_wallet))
}

async fn get_seqno(client: &TonClient, owner_address: &TonAddress) -> Result<i32> {
    let account_state = client.get_account_state(owner_address).await?;
    let seqno = account_state.block_id.seqno;
    Ok(seqno)
}

pub async fn generate_signed_message(
    client: &TonClient,
    key_pair: &KeyPair,
    jetton_contract_address: &str,
    receiver_address: &str,
    amount: u128,
) -> Result<Vec<u8>> {
    let (transfer_cell, user_jetton_wallet) = create_transfer_cell(
        client,
        key_pair,
        jetton_contract_address,
        receiver_address,
        amount,
    )
    .await?;

    log::info!("user jetton wallet: {}", user_jetton_wallet.to_base64_url());

    let payer_wallet = TonWallet::derive_default(WalletVersion::V4R2, &key_pair)?;

    let transfer = TransferMessage::new(&user_jetton_wallet, &1_000_000_000u32.into())
        .with_data(transfer_cell)
        .build()?;

    let transfer_cells: Vec<Arc<Cell>> = vec![Arc::new(transfer)];

    log::info!("Sending jetton to {}", receiver_address);

    log::info!("transfer cells {:?}", transfer_cells);

    let seqno = get_seqno(client, &payer_wallet.address).await?;
    log::info!("Seqno: {}", seqno);

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let body = payer_wallet.create_external_body(now + 60, seqno.try_into()?, transfer_cells)?;
    let signed = payer_wallet.sign_external_body(&body)?;

    let wrapped = payer_wallet.wrap_signed_body(signed, false)?;
    let boc = BagOfCells::from_root(wrapped);
    let tx = boc.serialize(true)?;
    Ok(tx)
}

pub async fn send_jetton(
    client: &TonClient,
    key_pair: &KeyPair,
    jetton_contract_address: &str,
    receiver_address: &str,
    amount: u128,
) -> Result<()> {
    let tx = generate_signed_message(
        client,
        key_pair,
        jetton_contract_address,
        receiver_address,
        amount,
    )
    .await?;

    let base_64 = BASE64_STANDARD.encode(&tx);
    log::info!("Sending raw message: {:?}", base_64);

    match client.send_raw_message_return_hash(tx.as_slice()).await {
        Ok(hash) => println!("Transaction hash: {:?}", hash),
        Err(err) => {
            log::error!("Failed to send raw message: {:?}", err);
            return Err(anyhow::anyhow!("Failed to send raw message: {:?}", err));
        }
    }

    Ok(())
}
