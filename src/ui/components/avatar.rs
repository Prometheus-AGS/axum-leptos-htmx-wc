//! Avatar component with image and fallback support.

use leptos::prelude::*;

/// Avatar component for displaying user images.
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <Avatar src="/images/user.jpg" alt="User" fallback="JD" />
/// }
/// ```
#[component]
pub fn Avatar(
    /// Image source URL.
    #[prop(default = "")]
    src: &'static str,
    /// Alt text for the image.
    #[prop(default = "Avatar")]
    alt: &'static str,
    /// Fallback text (initials) when image fails to load.
    #[prop(default = "")]
    fallback: &'static str,
    /// Size class (e.g., "h-10 w-10").
    #[prop(default = "h-10 w-10")]
    size: &'static str,
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let container_classes = format!(
        "relative flex shrink-0 overflow-hidden rounded-full {size} {class}"
    );

    let has_src = !src.is_empty();

    view! {
        <span class=container_classes>
            {if has_src {
                view! {
                    <img
                        class="aspect-square h-full w-full object-cover"
                        src=src
                        alt=alt
                    />
                }.into_any()
            } else {
                view! {
                    <span class="flex h-full w-full items-center justify-center rounded-full bg-panel text-textMuted text-sm font-medium">
                        {fallback}
                    </span>
                }.into_any()
            }}
        </span>
    }
}
