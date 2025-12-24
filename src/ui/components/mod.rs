//! ShadCN-style reusable UI components.
//!
//! This module provides a set of composable, accessible UI components
//! inspired by shadcn/ui, rendered via Leptos SSR.
//!
//! # Components
//!
//! - [`Button`]: Clickable button with variants
//! - [`Card`], [`CardHeader`], [`CardContent`], [`CardFooter`]: Card container
//! - [`Input`]: Text input field
//! - [`Badge`]: Status badge/tag
//! - [`Avatar`]: User avatar with fallback
//! - [`ScrollArea`]: Scrollable container
//! - [`Separator`]: Visual separator line
//! - [`icons`]: SVG icon components

mod avatar;
mod badge;
mod button;
mod card;
mod icons;
mod input;
mod scroll_area;
mod separator;

pub use badge::{Badge, BadgeVariant};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::{Card, CardContent, CardHeader};
pub use icons::*;
