//! Scrollable area component.

use leptos::prelude::*;

/// Scrollable container component.
///
/// Provides a styled scrollable area with custom scrollbar styling.
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <ScrollArea class="h-[400px]">
///         // Long content here
///     </ScrollArea>
/// }
/// ```
#[component]
pub fn ScrollArea(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Scrollable content.
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "relative overflow-auto scrollbar-thin scrollbar-thumb-panelBorder \
         scrollbar-track-transparent {}",
        class
    );

    view! {
        <div class=classes>
            {children()}
        </div>
    }
}
