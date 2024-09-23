use crate::storage::Storage;
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use tokio::sync::Mutex;
use yew_chat::prelude::{ReceiveError, ReceiveResponse, SendError, SendRequest};

#[utoipa::path(
    post,
    operation_id = "send",
    tag = "chat",
    path = "/send/{channel}",
    context_path = "/api/v1/chat",
    request_body = SendRequest,
    responses(
        (status = 200, description = "Message sent successfully", body = ()),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn send_message(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Path(channel): Path<String>,
    Json(message): Json<SendRequest>,
) -> Result<Json<()>, Json<SendError>> {
    let message = message.message;
    repository
        .lock()
        .await
        .send_message(&channel, message)
        .await
        .map(|_| Json(()))
        .map_err(Json)
}

#[utoipa::path(
    get,
    operation_id = "receive",
    tag = "chat",
    path = "/receive/{channel}",
    context_path = "/api/v1/chat",
    responses(
        (status = 200, description = "Messages received successfully", body = ReceiveResponse),
        (status = 400, description = "Invalid request data"),
    )
)]
pub async fn receive_messages(
    State(repository): State<Arc<Mutex<dyn Storage>>>,
    Path(channel): Path<String>,
) -> Result<Json<ReceiveResponse>, Json<ReceiveError>> {
    let lock = repository.lock().await;

    lock.receive_messages(&channel)
        .await
        .map(|messages| Json(ReceiveResponse { messages }))
        .map_err(Json)
}
