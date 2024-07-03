use crate::routes::v2::claim::{ClaimV2Request, ClaimV2Response};
use axum::{http::StatusCode, Json};

#[cfg(feature = "ton")]
use crate::ton::{create_key_pair, create_testnet_client, generate_signed_message};

#[cfg(feature = "ton")]
pub async fn claim_tokens_service(
    payload: ClaimV2Request,
) -> Result<Json<ClaimV2Response>, (StatusCode, String)> {
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

    match generate_signed_message(
        &client,
        &key_pair,
        &contract_address,
        &payload.address,
        payload.amount as u128,
    )
    .await
    {
        Ok(tx) => {
            let b64_tx = BASE64_STANDARD.encode(&tx);
            log::info!("Generated signed message: {:?}", tx);

            let response = ClaimV2Response {
                success: true,
                raw_transaction: b64_tx,
                destination: payload.address.clone(),
            };

            Ok(Json(response))
        }
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}
