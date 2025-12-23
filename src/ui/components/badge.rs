//! Badge component for status indicators and tags.

use leptos::prelude::*;

/// Badge visual variant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// Default badge style.
    #[default]
    Default,
    /// Success/positive badge.
    Success,
    /// Warning badge.
    Warning,
    /// Error/destructive badge.
    Error,
    /// Outline badge.
    Outline,
    /// Secondary badge.
    Secondary,
}

impl BadgeVariant {
    /// Get CSS classes for this variant.
    #[must_use]
    pub fn classes(self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-white",
            Self::Success => "bg-success text-white",
            Self::Warning => "bg-warning text-black",
            Self::Error => "bg-danger text-white",
            Self::Outline => "border border-panelBorder bg-transparent text-textPrimary",
            Self::Secondary => "bg-panel text-textPrimary border border-panelBorder",
        }
    }
}

/// Badge component for displaying status or labels.
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <Badge variant=BadgeVariant::Success>"Active"</Badge>
///     <Badge variant=BadgeVariant::Warning>"Pending"</Badge>
/// }
/// ```
#[component]
pub fn Badge(
    /// Badge variant.
    #[prop(default = BadgeVariant::Default)]
    variant: BadgeVariant,
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Badge content.
    children: Children,
) -> impl IntoView {
    let base_classes = "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold \
                        transition-colors";

    let classes = format!("{} {} {}", base_classes, variant.classes(), class);

    view! {
        <span class=classes>
            {children()}
        </span>
    }
}
