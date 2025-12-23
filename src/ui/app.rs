//! Main application component and routing.

use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

use crate::ui::chat::ChatShell;
use crate::ui::components::{Button, ButtonVariant, Card, CardContent, CardHeader, SparklesIcon};

/// Main application component.
///
/// Renders the complete application shell with routing.
#[component]
pub fn App() -> impl IntoView {
    view! {
        <!doctype html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="description" content="Agentic Streaming LLM Application"/>

                <title>"Prometheus - AI Assistant"</title>

                // Tauri-friendly: local scripts only (no CDN)
                <script src="/static/vendor/htmx-2.0.8.min.js"></script>
                <script defer src="/static/vendor/alpine.min.js"></script>

                // Application bundle
                <script type="module" src="/static/main.js"></script>
                <link rel="stylesheet" href="/static/app.css"/>
            </head>

            <body class="min-h-screen bg-background text-textPrimary antialiased">
                <div id="app-shell" class="flex flex-col min-h-screen">
                    <Header/>
                    <main id="app" class="flex-1 container mx-auto px-4 py-6 max-w-5xl">
                        <Router>
                            <Routes fallback=NotFoundPage>
                                <Route path=path!("") view=ChatPage/>
                                <Route path=path!("about") view=AboutPage/>
                            </Routes>
                        </Router>
                    </main>
                    <Footer/>
                </div>
            </body>
        </html>
    }
}

/// Application header with navigation.
#[component]
fn Header() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 w-full border-b border-panelBorder bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
            <div class="container mx-auto flex h-14 items-center justify-between px-4 max-w-5xl">
                <a href="/" class="flex items-center gap-2 font-semibold">
                    <SparklesIcon class="h-5 w-5 text-primary" />
                    <span class="text-lg">"Prometheus"</span>
                </a>

                <nav class="flex items-center gap-6" hx-boost="true">
                    <a href="/" class="text-sm text-textMuted hover:text-textPrimary transition-colors">
                        "Chat"
                    </a>
                    <a href="/about" class="text-sm text-textMuted hover:text-textPrimary transition-colors">
                        "About"
                    </a>
                </nav>
            </div>
        </header>
    }
}

/// Footer component.
#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-panelBorder py-4">
            <div class="container mx-auto px-4 max-w-5xl">
                <p class="text-xs text-textMuted text-center">
                    "Powered by Axum + Leptos + HTMX + Web Components"
                </p>
            </div>
        </footer>
    }
}

/// Main chat page.
#[component]
fn ChatPage() -> impl IntoView {
    view! {
        <ChatShell
            title="AI Assistant"
            stream_url="/stream"
            session_id=""
        />
    }
}

/// About page.
#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <Card>
                <CardHeader>
                    <h1 class="text-2xl font-bold">"About Prometheus"</h1>
                </CardHeader>
                <CardContent class="space-y-4">
                    <p>
                        "Prometheus is an agentic streaming LLM application that demonstrates
                        a modern architecture for building AI-powered applications."
                    </p>

                    <div class="grid gap-4 md:grid-cols-2">
                        <FeatureCard
                            title="Tool-First Design"
                            description="Always-on tool use with MCP integration for dynamic tool discovery and execution."
                        />
                        <FeatureCard
                            title="Streaming Native"
                            description="First-class streaming for tokens, tool calls, and results with AG-UI events."
                        />
                        <FeatureCard
                            title="HTML-Centric"
                            description="HTMX + Web Components + Alpine.js for a lightweight, inspectable UI."
                        />
                        <FeatureCard
                            title="Tauri Ready"
                            description="No CDN scripts, local assets only - runs as web, desktop, or mobile."
                        />
                    </div>

                    <div class="pt-4">
                        <a href="/">
                            <Button variant=ButtonVariant::Primary>
                                "Start Chatting"
                            </Button>
                        </a>
                    </div>
                </CardContent>
            </Card>
        </div>
    }
}

/// Feature card for the about page.
#[component]
fn FeatureCard(
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="p-4 rounded-lg border border-panelBorder bg-panel/50">
            <h3 class="font-semibold mb-2">{title}</h3>
            <p class="text-sm text-textMuted">{description}</p>
        </div>
    }
}

/// 404 Not Found page.
#[component]
fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-20">
            <h1 class="text-4xl font-bold mb-4">"404"</h1>
            <p class="text-textMuted mb-6">"Page not found"</p>
            <a href="/">
                <Button variant=ButtonVariant::Primary>
                    "Go Home"
                </Button>
            </a>
        </div>
    }
}
