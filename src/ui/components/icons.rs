//! SVG icon components.
//!
//! Icons are rendered inline as SVG elements for optimal performance
//! and styling flexibility.

use leptos::prelude::*;

/// Common icon size class.
const ICON_SIZE: &str = "h-4 w-4";

/// Send/arrow-right icon.
#[component]
pub fn SendIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <line x1="22" y1="2" x2="11" y2="13" />
            <polygon points="22 2 15 22 11 13 2 9 22 2" />
        </svg>
    }
}

/// Loader/spinner icon.
#[component]
pub fn LoaderIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {} animate-spin", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <path d="M21 12a9 9 0 1 1-6.219-8.56" />
        </svg>
    }
}

/// Copy/clipboard icon.
#[component]
pub fn CopyIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
        </svg>
    }
}

/// Check/success icon.
#[component]
pub fn CheckIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <polyline points="20 6 9 17 4 12" />
        </svg>
    }
}

/// X/close icon.
#[component]
pub fn XIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
    }
}

/// Menu/hamburger icon.
#[component]
pub fn MenuIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <line x1="3" y1="12" x2="21" y2="12" />
            <line x1="3" y1="6" x2="21" y2="6" />
            <line x1="3" y1="18" x2="21" y2="18" />
        </svg>
    }
}

/// Tool/wrench icon.
#[component]
pub fn ToolIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
        </svg>
    }
}

/// User icon.
#[component]
pub fn UserIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" />
            <circle cx="12" cy="7" r="4" />
        </svg>
    }
}

/// Bot/robot icon.
#[component]
pub fn BotIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <rect x="3" y="11" width="18" height="10" rx="2" />
            <circle cx="12" cy="5" r="2" />
            <path d="M12 7v4" />
            <line x1="8" y1="16" x2="8" y2="16" />
            <line x1="16" y1="16" x2="16" y2="16" />
        </svg>
    }
}

/// Sparkles/AI icon.
#[component]
pub fn SparklesIcon(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let classes = format!("{} {}", ICON_SIZE, class);

    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=classes
        >
            <path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z" />
            <path d="M5 3v4" />
            <path d="M19 17v4" />
            <path d="M3 5h4" />
            <path d="M17 19h4" />
        </svg>
    }
}
