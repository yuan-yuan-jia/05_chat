use axum::extract::State;
use axum::response::sse::Event;
use axum::response::Sse;
use axum::Extension;
use axum_extra::{headers, TypedHeader};
use chat_core::User;
use futures::Stream;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt;
use tracing::info;

use crate::notif::AppEvent;
use crate::AppState;

const CHANNEL_CAPACITY: usize = 256;

pub(crate) async fn sse_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream

    let user_id = user.id as u64;
    let users = &state.users;
    let rx = if let Some(tx) = users.get(&user_id) {
        tx.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(CHANNEL_CAPACITY);
        state.users.insert(user_id, tx);
        rx
    };
    info!("User {} subscribed", user_id);
    let stream = BroadcastStream::new(rx).filter_map(|v| v.ok()).map(|v| {
        let name = match v.as_ref() {
            AppEvent::NewChat(_) => "NewChat",
            AppEvent::AddToChat(_) => "AddToChat",
            AppEvent::RemoveFromChat(_) => "RemoveFromChat",
            AppEvent::NewMessage(_) => "NewMessage",
        };
        let v = serde_json::to_string(&v).expect("Failed to serialize event");
        Ok(Event::default().data(v).event(name))
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
