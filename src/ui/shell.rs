use leptos::ssr::render_to_string;
use leptos::view;

const INDEX_BODY: &str = r##"
    <div id="app-shell" class="flex flex-col h-screen overflow-hidden">
        <header class="sticky top-0 z-50 w-full bg-surfaceContainer backdrop-blur shadow-sm shrink-0">
            <div class="container mx-auto flex h-14 md:h-16 items-center justify-between px-4 md:px-6 max-w-5xl">
                <a href="/" class="flex items-center gap-2 md:gap-3 font-semibold hover:opacity-80 transition-opacity">
                    <svg class="h-5 w-5 md:h-6 md:w-6 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                    </svg>
                    <span class="text-base md:text-lg">Prometheus</span>
                </a>
                <div class="flex items-center gap-1 md:gap-2">
                    <nav class="flex items-center gap-1" hx-boost="true" hx-target="#app-shell" hx-select="#app-shell" hx-swap="outerHTML">
                        <a href="/about" class="px-3 py-2 rounded-xl text-sm text-textSecondary hover:text-textPrimary hover:bg-surface transition-all">About</a>
                    </nav>
                    <theme-switcher></theme-switcher>
                </div>
            </div>
        </header>
        
        <main id="app" class="flex-1 overflow-y-auto container mx-auto px-4 md:px-6 py-4 md:py-8 max-w-5xl">
            <div class="flex h-full md:h-[calc(100vh-12rem)]">
                <!-- Conversation Sidebar -->
                <error-boundary>
                    <conversation-sidebar></conversation-sidebar>
                </error-boundary>
                
                <!-- Main Chat Area -->
                <div class="chat-shell flex flex-col flex-1 bg-surface md:rounded-3xl overflow-hidden md:shadow-lg" style="margin-left: 288px;">
                    <header class="flex items-center justify-between px-4 md:px-6 py-3 md:py-4 bg-surfaceContainer shrink-0">
                        <div class="flex items-center gap-2 md:gap-3">
                            <svg class="h-5 w-5 md:h-6 md:w-6 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                            </svg>
                            <h2 class="font-semibold text-base md:text-lg">AI Assistant</h2>
                        </div>
                        <div class="flex items-center gap-3">
                            <token-counter
                                x-bind:input-tokens="$store.chat.tokenUsage.input"
                                x-bind:output-tokens="$store.chat.tokenUsage.output"
                                x-bind:context-limit="$store.chat.tokenUsage.limit"
                                x-bind:cost="$store.chat.tokenUsage.cost"
                                x-bind:is-estimate="$store.chat.tokenUsage.isEstimate"
                                model-id="gpt-4o">
                            </token-counter>
                            <storage-health></storage-health>
                            <button
                                type="button"
                                class="p-2 rounded-xl hover:bg-surface transition-colors"
                                aria-label="Start new chat"
                                title="Start new chat"
                                x-on:click="
                                    $store.chat.sessionId = null;
                                    $store.chat.tokenUsage = {
                                        input: 0,
                                        output: 0,
                                        total: 0,
                                        limit: 128000,
                                        isEstimate: true,
                                        cost: 0
                                    };
                                    
                                    const chatStream = document.querySelector('chat-stream');
                                    if (chatStream) {
                                        chatStream.createNewConversation();
                                    }
                                "
                            >
                                <svg class="h-5 w-5 text-textPrimary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M12 5v14"/>
                                    <path d="M5 12h14"/>
                                </svg>
                            </button>
                        </div>
                    </header>
                    
                    <div class="flex-1 overflow-y-auto overflow-x-hidden">
                        <error-boundary>
                            <chat-stream class="block" stream-url="/stream"></chat-stream>
                        </error-boundary>
                    </div>
                
                <div class="p-3 md:p-5 bg-surfaceContainer shrink-0">
                    <form 
                        class="flex gap-2 md:gap-3"
                        hx-post="/api/chat"
                        hx-trigger="submit"
                        hx-swap="none"
                        hx-ext="json-enc"
                        hx-on::before-request="
                            const msg = this.querySelector('[name=message]').value;
                            const chatStream = document.querySelector('chat-stream');
                            const fileUpload = document.querySelector('file-upload');
                            
                            const attachedFiles = fileUpload ? fileUpload.getAttachedFiles() : [];
                            
                            if (chatStream && (msg.trim() || attachedFiles.length > 0)) {
                                chatStream.addUserMessage(msg, attachedFiles);
                            }
                            
                            if (fileUpload) {
                                fileUpload.clearFiles();
                            }
                            
                            const Alpine = window.Alpine;
                            const sessionInput = this.querySelector('[name=session_id]');
                            if (Alpine) {
                                const chatStore = Alpine.store('chat');
                                if (chatStore && chatStore.sessionId) {
                                    sessionInput.value = chatStore.sessionId;
                                } else {
                                    sessionInput.value = '';
                                }
                            }
                        "
                        hx-on::after-request="
                            const response = JSON.parse(event.detail.xhr.response);
                            const chatStream = document.querySelector('chat-stream');
                            
                            const Alpine = window.Alpine;
                            if (Alpine && response.session_id) {
                                const chatStore = Alpine.store('chat');
                                if (chatStore) {
                                    chatStore.sessionId = response.session_id;
                                }
                            }
                            
                            if (chatStream) {
                                chatStream.startStream(event.detail.xhr.response);
                            }
                            
                            this.reset();
                        "
                        x-data="{ message: '' }"
                    >
                        <input type="hidden" name="session_id" x-bind:value="$store.chat?.sessionId || ''">
                        
                        <file-upload class="relative"></file-upload>
                        
                        <div class="flex-1">
                            <textarea
                                name="message"
                                placeholder="Type your message..."
                                class="w-full min-h-[44px] md:min-h-[48px] max-h-[120px] md:max-h-[200px] px-4 md:px-5 py-3 md:py-3.5 rounded-xl md:rounded-2xl bg-surface text-textPrimary placeholder:text-textMuted resize-none focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 focus:ring-offset-surfaceContainer transition-shadow text-sm md:text-base"
                                rows="1"
                                x-model="message"
                                x-on:keydown.enter.prevent="if (!$event.shiftKey && message.trim()) { $el.form.requestSubmit() }"
                                x-on:input="$el.style.height = 'auto'; $el.style.height = Math.min($el.scrollHeight, window.innerWidth < 768 ? 120 : 200) + 'px'"
                                required
                            ></textarea>
                        </div>
                        <button 
                            type="submit"
                            class="shrink-0 h-11 w-11 md:h-12 md:w-12 rounded-xl md:rounded-2xl bg-primary text-white hover:bg-primaryMuted active:scale-95 flex items-center justify-center transition-all shadow-md hover:shadow-lg"
                            aria-label="Send message"
                        >
                            <svg class="h-5 w-5 md:h-6 md:w-6" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <line x1="22" y1="2" x2="11" y2="13"></line>
                                <polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
                            </svg>
                        </button>
                    </form>
                    <p class="text-xs text-textMuted mt-2 md:mt-3 text-center hidden md:block">Press Enter to send, Shift+Enter for new line</p>
                </div>
            </div>
            </div>
        </main>
        
        <footer class="bg-surfaceContainer py-3 md:py-6 shrink-0 hidden md:block">
            <div class="container mx-auto px-4 md:px-6 max-w-5xl">
                <p class="text-xs text-textMuted text-center">
                    Powered by Axum + Leptos + HTMX + Web Components
                </p>
            </div>
        </footer>
    </div>
"##;

const ABOUT_BODY: &str = r##"
    <div id="app-shell" class="flex flex-col h-screen overflow-hidden">
        <header class="sticky top-0 z-50 w-full bg-surfaceContainer backdrop-blur shadow-sm shrink-0">
            <div class="container mx-auto flex h-14 md:h-16 items-center justify-between px-4 md:px-6 max-w-5xl">
                <a href="/" class="flex items-center gap-2 md:gap-3 font-semibold hover:opacity-80 transition-opacity">
                    <svg class="h-5 w-5 md:h-6 md:w-6 text-primary" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/>
                    </svg>
                    <span class="text-base md:text-lg">Prometheus</span>
                </a>
                <div class="flex items-center gap-1 md:gap-2">
                    <nav class="flex items-center gap-1" hx-boost="true" hx-target="#app-shell" hx-select="#app-shell" hx-swap="outerHTML">
                        <a href="/about" class="px-3 py-2 rounded-xl text-sm text-textSecondary hover:text-textPrimary hover:bg-surface transition-all">About</a>
                    </nav>
                    <theme-switcher></theme-switcher>
                </div>
            </div>
        </header>
        
        <main id="app" class="flex-1 overflow-y-auto container mx-auto px-4 md:px-6 py-4 md:py-8 max-w-5xl">
            <div class="space-y-6">
                <div class="rounded-3xl bg-surface p-8 shadow-lg">
                    <h1 class="text-2xl font-bold mb-4">About Prometheus</h1>
                    <p class="text-textMuted mb-8">
                        Prometheus is an agentic streaming LLM application that demonstrates
                        a modern architecture for building AI-powered applications.
                    </p>
                    
                    <div class="grid gap-4 md:grid-cols-2">
                        <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                            <h3 class="font-semibold mb-2">🔧 Tool-First Design</h3>
                            <p class="text-sm text-textMuted">Always-on tool use with MCP integration for dynamic tool discovery and execution.</p>
                        </div>
                        <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                            <h3 class="font-semibold mb-2">⚡ Streaming Native</h3>
                            <p class="text-sm text-textMuted">First-class streaming for tokens, tool calls, and results with AG-UI events.</p>
                        </div>
                        <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                            <h3 class="font-semibold mb-2">🌐 HTML-Centric</h3>
                            <p class="text-sm text-textMuted">HTMX + Web Components + Alpine.js for a lightweight, inspectable UI.</p>
                        </div>
                        <div class="p-5 rounded-2xl bg-surfaceVariant hover:bg-surfaceContainer transition-colors">
                            <h3 class="font-semibold mb-2">📦 Tauri Ready</h3>
                            <p class="text-sm text-textMuted">No CDN scripts, local assets only - runs as web, desktop, or mobile.</p>
                        </div>
                    </div>
                    
                    <div class="mt-8">
                        <a href="/" class="inline-flex items-center justify-center h-12 px-6 rounded-2xl bg-primary text-white hover:bg-primaryMuted active:scale-95 font-medium transition-all shadow-md hover:shadow-lg">
                            Start Chatting
                        </a>
                    </div>
                </div>
            </div>
        </main>
        
        <footer class="bg-surfaceContainer py-3 md:py-6 shrink-0 hidden md:block">
            <div class="container mx-auto px-4 md:px-6 max-w-5xl">
                <p class="text-xs text-textMuted text-center">
                    Powered by Axum + Leptos + HTMX + Web Components
                </p>
            </div>
        </footer>
    </div>
"##;

fn render_document(title: &str, body: &str) -> String {
    let title = title.to_string();
    let body = body.to_string();

    let html = render_to_string(move || {
        view! {
            <html lang="en">
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <meta name="description" content="Agentic Streaming LLM Application"/>
                    <title>{title}</title>
                    <script src="/static/vendor/htmx-2.0.8.min.js"></script>
                    <script src="/static/vendor/htmx-json-enc.js"></script>
                    <script src="/static/vendor/htmx-sse.js"></script>
                    <script defer src="/static/vendor/alpine.min.js"></script>
                    <script type="module" src="/static/main.js"></script>
                    <link rel="stylesheet" href="/static/app.css"/>
                </head>
                <body class="min-h-screen bg-background text-textPrimary antialiased" inner_html=body></body>
            </html>
        }
    });

    format!("<!DOCTYPE html>{html}")
}

pub fn render_index() -> String {
    render_document("Chat - Prometheus", INDEX_BODY)
}

pub fn render_about() -> String {
    render_document("About - Prometheus", ABOUT_BODY)
}
