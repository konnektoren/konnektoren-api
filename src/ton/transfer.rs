use anyhow::Result;
use num_bigint::BigUint;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tonlib::address::TonAddress;
use tonlib::cell::{BagOfCells, Cell};
use tonlib::client::{TonClient, TonClientInterface};
use tonlib::contract::{
    JettonMasterContract, JettonWalletContract, TonContract, TonContractFactory,
    TonContractInterface,
};
use tonlib::message::{JettonTransferMessage, TransferMessage};
use tonlib::mnemonic::KeyPair;
use tonlib::wallet::{TonWallet, WalletVersion};
use uuid::Uuid;

async fn get_seqno(client: &TonClient, owner_address: &TonAddress) -> Result<i32> {
    let account_state = client.get_account_state(owner_address).await?;
    let seqno = account_state.block_id.seqno;
    Ok(seqno)
}

async fn log_wallet_info(
    client: &TonClient,
    wallet_address: &TonAddress,
    actor: &str,
) -> Result<()> {
    let account_state = client.get_account_state(wallet_address).await?;
    let balance = account_state.balance;
    log::info!(
        "{} Account Info: {:?}, Balance: {}",
        actor,
        account_state,
        balance
    );
    Ok(())
}

async fn log_jetton_wallet_info(
    jetton_contract: &TonContract,
    wallet_address: &TonAddress,
    contract_factory: &TonContractFactory,
    actor: &str,
) -> Result<()> {
    let jetton_wallet_addr = jetton_contract.get_wallet_address(&wallet_address).await?;
    log::info!(
        "Jetton Wallet Address ({}): {:?}",
        actor,
        jetton_wallet_addr
    );
    let jetton_wallet = contract_factory.get_contract(&jetton_wallet_addr);
    let balance = jetton_wallet.get_wallet_data().await?.balance;
    log::info!("Balance ({}): {:?}", actor, balance);
    Ok(())
}

async fn log_jetton_info(jetton_contract: &TonContract) -> Result<()> {
    let jetton_data = jetton_contract.get_jetton_data().await?;
    let total_supply = jetton_data.total_supply;
    log::info!("Total Supply: {:?}", total_supply);
    Ok(())
}

///
/// Transfer Jetton Token from one account to another
///
/// # Arguments
///
/// * `client` - TonClient
/// * `key_pair` - KeyPair
/// * `jetton_contract_address` - Jetton Token Contract Address
/// * `sender_address` - Sender Address
/// * `receiver_address` - Receiver Address
/// * `amount` - Amount to transfer
///
/// https://docs.ton.org/develop/dapps/asset-processing/jettons#jetton-wallets-communication-overview
///
pub async fn transfer_jetton_token(
    client: &TonClient,
    key_pair: &KeyPair,
    jetton_contract_address: &str,
    sender_address: &str,
    receiver_address: &str,
    amount: u128,
) -> Result<()> {
    log::info!("Jetton Contract {}", jetton_contract_address);

    log::info!(
        "Transferring from {} (Bob) to {} (Alice)",
        sender_address,
        receiver_address
    );

    let sender_address = TonAddress::from_base64_url(sender_address)?;
    let receiver_address = TonAddress::from_base64_url(receiver_address)?;

    log_wallet_info(client, &sender_address, "bob").await?;
    log_wallet_info(client, &receiver_address, "alice").await?;

    // Get the latest seqno
    let seqno = get_seqno(client, &sender_address).await?;
    log::info!("Seqno: {}", seqno);

    let payer_wallet = TonWallet::derive_default(WalletVersion::V4R2, &key_pair)?;

    let factory = TonContractFactory::builder(client).build().await?;

    let jetton_contract = factory.get_contract(&jetton_contract_address.parse()?);

    assert_eq!(
        jetton_contract_address,
        jetton_contract.address().to_string()
    );

    log_jetton_info(&jetton_contract).await?;

    let sender_jetton_wallet_addr = jetton_contract.get_wallet_address(&sender_address).await?;

    log_jetton_wallet_info(&jetton_contract, &sender_address, &factory, "Bob").await?;
    log_jetton_wallet_info(&jetton_contract, &receiver_address, &factory, "Alice").await?;

    let sender_jetton_wallet = factory.get_contract(&sender_jetton_wallet_addr);

    let jetton_amount = BigUint::from(amount);

    // Generate a unique query ID
    let query_id = Uuid::new_v4().as_u128() as u64;

    let receiver_jetton_wallet_addr = jetton_contract
        .get_wallet_address(&receiver_address)
        .await?;
    log::info!(
        "Receiver (Alice) Jetton Wallet Address: {:?}",
        receiver_jetton_wallet_addr
    );

    let receiver_wallet = factory.get_contract(&receiver_jetton_wallet_addr);
    let receiver_balance = receiver_wallet.get_wallet_data().await?.balance;
    log::info!("Receiver (Alice) Balance: {:?}", receiver_balance);

    let jetton_transfer = JettonTransferMessage::new(&receiver_address, &jetton_amount)
        .with_query_id(query_id)
        .with_response_destination(&sender_address)
        .build()?;

    let ton_amount = BigUint::from(1000000000u64); // 1 TON for example
    let transfer = TransferMessage::new(&receiver_wallet.address(), &ton_amount)
        //.with_data(jetton_transfer)
        .build()?;

    let transfer_cells: Vec<Arc<Cell>> = vec![Arc::new(transfer)];

    let seqno = get_seqno(client, &payer_wallet.address).await?;
    log::info!("Seqno: {}", seqno);

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let body = payer_wallet.create_external_body(now + 60, seqno.try_into()?, transfer_cells)?;
    let signed = payer_wallet.sign_external_body(&body)?;
    let wrapped = payer_wallet.wrap_signed_body(signed, false)?;
    let boc = BagOfCells::from_root(wrapped);
    let tx = boc.serialize(true)?;

    match client.send_raw_message_return_hash(tx.as_slice()).await {
        Ok(hash) => println!("Transaction hash: {:?}", hash),
        Err(err) => {
            log::error!("Failed to send raw message: {:?}", err);
            return Err(anyhow::anyhow!("Failed to send raw message: {:?}", err));
        }
    }

    Ok(())
}
