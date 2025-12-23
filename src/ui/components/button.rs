//! Button component with variants and sizes.

use leptos::prelude::*;

/// Button visual variant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Primary action button.
    #[default]
    Primary,
    /// Secondary action button.
    Secondary,
    /// Subtle ghost button.
    Ghost,
    /// Destructive action button.
    Destructive,
    /// Outline button.
    Outline,
    /// Link-style button.
    Link,
}

impl ButtonVariant {
    /// Get CSS classes for this variant.
    #[must_use]
    pub fn classes(self) -> &'static str {
        match self {
            Self::Primary => "bg-primary text-white hover:bg-primaryMuted",
            Self::Secondary => "bg-panel text-textPrimary border border-panelBorder hover:bg-panelBorder",
            Self::Ghost => "bg-transparent text-textPrimary hover:bg-panel",
            Self::Destructive => "bg-danger text-white hover:bg-red-600",
            Self::Outline => "bg-transparent border border-panelBorder text-textPrimary hover:bg-panel",
            Self::Link => "bg-transparent text-primary underline-offset-4 hover:underline",
        }
    }
}

/// Button size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// Small button.
    Sm,
    /// Medium button (default).
    #[default]
    Md,
    /// Large button.
    Lg,
    /// Icon-only button.
    Icon,
}

impl ButtonSize {
    /// Get CSS classes for this size.
    #[must_use]
    pub fn classes(self) -> &'static str {
        match self {
            Self::Sm => "h-8 px-3 text-xs",
            Self::Md => "h-10 px-4 text-sm",
            Self::Lg => "h-12 px-6 text-base",
            Self::Icon => "h-10 w-10",
        }
    }
}

/// ShadCN-style button component.
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <Button variant=ButtonVariant::Primary size=ButtonSize::Md>
///         "Click me"
///     </Button>
/// }
/// ```
#[component]
pub fn Button(
    /// Button variant.
    #[prop(default = ButtonVariant::Primary)]
    variant: ButtonVariant,
    /// Button size.
    #[prop(default = ButtonSize::Md)]
    size: ButtonSize,
    /// Whether the button is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Button type attribute.
    #[prop(default = "button")]
    button_type: &'static str,
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Button content.
    children: Children,
) -> impl IntoView {
    let base_classes = "inline-flex items-center justify-center rounded-lg font-medium \
                        transition-colors focus-visible:outline-none focus-visible:ring-2 \
                        focus-visible:ring-primary focus-visible:ring-offset-2 \
                        disabled:pointer-events-none disabled:opacity-50";

    let classes = format!(
        "{} {} {} {}",
        base_classes,
        variant.classes(),
        size.classes(),
        class
    );

    view! {
        <button type=button_type class=classes disabled=disabled>
            {children()}
        </button>
    }
}
