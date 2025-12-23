//! Input component for text fields.

use leptos::prelude::*;

/// Text input component.
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <Input
///         input_type="text"
///         placeholder="Enter your message..."
///         name="message"
///     />
/// }
/// ```
#[component]
pub fn Input(
    /// Input type (text, email, password, etc.).
    #[prop(default = "text")]
    input_type: &'static str,
    /// Placeholder text.
    #[prop(default = "")]
    placeholder: &'static str,
    /// Input name attribute.
    #[prop(default = "")]
    name: &'static str,
    /// Input ID attribute.
    #[prop(default = "")]
    id: &'static str,
    /// Whether the input is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Whether the input is required.
    #[prop(default = false)]
    required: bool,
    /// Default value.
    #[prop(default = "")]
    value: &'static str,
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Autocomplete attribute.
    #[prop(default = "off")]
    autocomplete: &'static str,
) -> impl IntoView {
    let base_classes = "flex h-10 w-full rounded-lg border border-panelBorder bg-background \
                        px-3 py-2 text-sm text-textPrimary placeholder:text-textMuted \
                        focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary \
                        focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50";

    let classes = format!("{} {}", base_classes, class);

    view! {
        <input
            type=input_type
            class=classes
            placeholder=placeholder
            name=name
            id=id
            disabled=disabled
            required=required
            value=value
            autocomplete=autocomplete
        />
    }
}

/// Textarea component for multi-line input.
#[component]
pub fn Textarea(
    /// Placeholder text.
    #[prop(default = "")]
    placeholder: &'static str,
    /// Input name attribute.
    #[prop(default = "")]
    name: &'static str,
    /// Input ID attribute.
    #[prop(default = "")]
    id: &'static str,
    /// Number of rows.
    #[prop(default = 3)]
    rows: u32,
    /// Whether the input is disabled.
    #[prop(default = false)]
    disabled: bool,
    /// Whether the input is required.
    #[prop(default = false)]
    required: bool,
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
) -> impl IntoView {
    let base_classes = "flex min-h-[80px] w-full rounded-lg border border-panelBorder bg-background \
                        px-3 py-2 text-sm text-textPrimary placeholder:text-textMuted \
                        focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary \
                        focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 \
                        resize-none";

    let classes = format!("{} {}", base_classes, class);

    view! {
        <textarea
            class=classes
            placeholder=placeholder
            name=name
            id=id
            rows=rows
            disabled=disabled
            required=required
        />
    }
}
