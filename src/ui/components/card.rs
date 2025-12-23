//! Card component with header, content, and footer.

use leptos::prelude::*;

/// Card container component.
///
/// # Example
///
/// ```rust,ignore
/// view! {
///     <Card>
///         <CardHeader>
///             <h3>"Title"</h3>
///         </CardHeader>
///         <CardContent>
///             <p>"Content goes here"</p>
///         </CardContent>
///         <CardFooter>
///             <Button>"Action"</Button>
///         </CardFooter>
///     </Card>
/// }
/// ```
#[component]
pub fn Card(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Card content.
    children: Children,
) -> impl IntoView {
    let classes = format!(
        "rounded-xl border border-panelBorder bg-panel text-textPrimary shadow-sm {}",
        class
    );

    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

/// Card header section.
#[component]
pub fn CardHeader(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Header content.
    children: Children,
) -> impl IntoView {
    let classes = format!("flex flex-col space-y-1.5 p-6 {}", class);

    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

/// Card content section.
#[component]
pub fn CardContent(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Content.
    children: Children,
) -> impl IntoView {
    let classes = format!("p-6 pt-0 {}", class);

    view! {
        <div class=classes>
            {children()}
        </div>
    }
}

/// Card footer section.
#[component]
pub fn CardFooter(
    /// Additional CSS classes.
    #[prop(default = "")]
    class: &'static str,
    /// Footer content.
    children: Children,
) -> impl IntoView {
    let classes = format!("flex items-center p-6 pt-0 {}", class);

    view! {
        <div class=classes>
            {children()}
        </div>
    }
}
