use crate::routes::v1::claim::ClaimRequest;
use axum::{http::StatusCode, Json};
use std::env;

#[cfg(feature = "ton")]
use crate::ton::{create_key_pair, create_testnet_client, send_jetton, transfer_jetton_token};

#[cfg(feature = "ton")]
pub async fn claim_tokens_service(
    payload: ClaimRequest,
) -> Result<Json<&'static str>, (StatusCode, String)> {
    if payload.request_type != "claim" {
        return Err((StatusCode::BAD_REQUEST, "Invalid request type".into()));
    }

    log::info!("Received claim request: {:?}", payload);

    let client = match create_testnet_client().await {
        Ok(client) => client,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    };

    let mnemonic = env::var("MNEMONIC").unwrap();
    let key_pair = create_key_pair(&mnemonic).await.unwrap();

    let contract_address = env::var("CONTRACT_ADDRESS").unwrap();
    let faucet_address = env::var("FAUCET_ADDRESS").unwrap();

    match send_jetton(
        &client,
        &key_pair,
        &contract_address,
        &payload.address,
        payload.amount as u128,
    )
    .await
    {
        Ok(_) => Ok(Json("Token claimed successfully")),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}
