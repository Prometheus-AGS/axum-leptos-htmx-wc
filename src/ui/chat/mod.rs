//! Chat-specific UI components.
//!
//! These components provide the layout and structure for the chat interface,
//! designed to work with Web Components for streaming content.

mod header;
mod input_area;
mod message_list;
mod shell;

pub use header::ChatHeader;
pub use input_area::ChatInputArea;
pub use message_list::ChatMessageList;
pub use shell::ChatShell;
