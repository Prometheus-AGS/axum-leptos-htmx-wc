//! Chat header component.

use leptos::prelude::*;

use crate::ui::components::{Badge, BadgeVariant, SparklesIcon};

/// Chat header with title and status.
#[component]
pub fn ChatHeader(
    /// Title displayed in the header.
    #[prop(default = "Chat")]
    title: &'static str,
) -> impl IntoView {
    view! {
        <header class="flex items-center justify-between px-4 py-3 border-b border-panelBorder bg-panel/50 backdrop-blur-sm">
            <div class="flex items-center gap-2">
                <SparklesIcon class="h-5 w-5 text-primary" />
                <h2 class="font-semibold text-lg">{title}</h2>
            </div>

            <div class="flex items-center gap-2">
                <Badge variant=BadgeVariant::Secondary>
                    <span id="chat-status" class="text-xs">"Ready"</span>
                </Badge>
            </div>
        </header>
    }
}
