//! Chat message list component.

use leptos::prelude::*;

/// Container for chat messages.
///
/// This component renders the Web Component `<chat-stream>` which handles
/// SSE streaming and message rendering on the client side.
#[component]
pub fn ChatMessageList(
    /// SSE stream URL.
    #[prop(default = "/stream")]
    stream_url: &'static str,
    /// Session ID.
    #[prop(default = "")]
    session_id: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex-1 overflow-hidden">
            // The chat-stream Web Component handles SSE streaming and rendering
            <chat-stream
                class="block h-full"
                stream-url=stream_url
                session-id=session_id
                aria-live="polite"
                aria-label="Chat messages"
            />
        </div>
    }
}
