use crate::uar::domain::events::NormalizedEvent;
use axum::response::sse::{Event, Sse};
use futures::{Stream, StreamExt};
use std::convert::Infallible;
use std::time::Duration;

pub fn build_sse_response<S>(stream: S) -> Sse<impl Stream<Item = Result<Event, Infallible>> + Send>
where
    S: Stream<Item = NormalizedEvent> + Send + 'static,
{
    let stream = stream.map(|event| {
        let json = serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string());

        let mut sse_event = Event::default().data(json);

        // Add event type if needed for client routing (e.g. HTMX sse-swap)
        // For general usage, we might just use 'message' or inspect payload
        if let NormalizedEvent::Error { .. } = event {
            sse_event = sse_event.event("error");
        } else if let NormalizedEvent::RunDone { .. } = event {
            sse_event = sse_event.event("done");
        } else {
            sse_event = sse_event.event("message");
        }

        Ok(sse_event)
    });

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}
