//! Chat shell layout component.

use leptos::prelude::*;

use super::{ChatHeader, ChatInputArea, ChatMessageList};

/// Main chat shell component.
///
/// Provides the complete chat interface layout with:
/// - Header with title and actions
/// - Scrollable message area
/// - Input area for new messages
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <ChatShell
///         title="AI Assistant"
///         stream_url="/api/chat/stream"
///         session_id="abc123"
///     />
/// }
/// ```
#[component]
pub fn ChatShell(
    /// Title displayed in the header.
    #[prop(default = "Chat")]
    title: &'static str,
    /// SSE stream URL for the chat.
    #[prop(default = "/stream")]
    stream_url: &'static str,
    /// Optional session ID.
    #[prop(default = "")]
    session_id: &'static str,
) -> impl IntoView {
    view! {
        <div class="chat-shell flex flex-col h-[calc(100vh-6rem)] bg-panel border border-panelBorder rounded-2xl overflow-hidden">
            <ChatHeader title=title />

            <ChatMessageList stream_url=stream_url session_id=session_id />

            <ChatInputArea session_id=session_id />
        </div>
    }
}
