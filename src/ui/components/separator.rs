//! Separator component for visual division.

use leptos::prelude::*;

/// Separator orientation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    /// Horizontal separator (default).
    #[default]
    Horizontal,
    /// Vertical separator.
    Vertical,
}

/// Visual separator line component.
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <div class="space-y-4">
///         <p>"Above"</p>
///         <Separator />
///         <p>"Below"</p>
///     </div>
/// }
/// ```
#[component]
pub fn Separator(
    /// Separator orientation.
    #[prop(default = SeparatorOrientation::Horizontal)]
    orientation: SeparatorOrientation,
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let base_classes = "shrink-0 bg-panelBorder";

    let orientation_classes = match orientation {
        SeparatorOrientation::Horizontal => "h-[1px] w-full",
        SeparatorOrientation::Vertical => "h-full w-[1px]",
    };

    let classes = format!("{} {} {}", base_classes, orientation_classes, class);

    view! {
        <div role="separator" class=classes />
    }
}
