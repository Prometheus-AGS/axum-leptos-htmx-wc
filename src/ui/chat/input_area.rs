//! Chat input area component.

use leptos::prelude::*;

use crate::ui::components::{Button, ButtonSize, ButtonVariant, SendIcon};

/// Chat message input area with HTMX form submission.
#[component]
pub fn ChatInputArea(
    /// Session ID for the chat.
    #[prop(default = "")]
    session_id: &'static str,
) -> impl IntoView {
    view! {
        <div class="border-t border-panelBorder p-4 bg-panel/50 backdrop-blur-sm">
            <form
                class="flex gap-2"
                hx-post="/api/chat"
                hx-trigger="submit"
                hx-swap="none"
                hx-on--after-request="this.reset(); document.querySelector('chat-stream')?.startStream(event.detail.xhr.response)"
                x-data="{ message: '', loading: false }"
            >
                <input type="hidden" name="session_id" value=session_id />

                <div class="flex-1 relative">
                    <textarea
                        name="message"
                        placeholder="Type your message..."
                        class="w-full min-h-[44px] max-h-[200px] px-4 py-3 pr-12 rounded-xl \
                               border border-panelBorder bg-background text-textPrimary \
                               placeholder:text-textMuted resize-none \
                               focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
                        rows="1"
                        x-model="message"
                        x-on:keydown.enter.prevent="if (!$event.shiftKey && message.trim()) { $el.form.requestSubmit() }"
                        x-on:input="$el.style.height = 'auto'; $el.style.height = Math.min($el.scrollHeight, 200) + 'px'"
                        required
                    />
                </div>

                <Button
                    variant=ButtonVariant::Primary
                    size=ButtonSize::Icon
                    button_type="submit"
                    class="shrink-0 h-11 w-11 rounded-xl"
                >
                    <SendIcon class="h-5 w-5" />
                </Button>
            </form>

            <p class="text-xs text-textMuted mt-2 text-center">
                "Press Enter to send, Shift+Enter for new line"
            </p>
        </div>
    }
}
