//! UI components and layouts.
//!
//! This module provides Leptos SSR components for rendering the application shell,
//! following ShadCN-UI design principles.
//!
//! # Structure
//!
//! - [`app`]: Main application component and routing
//! - [`components`]: Reusable ShadCN-style UI components
//! - [`chat`]: Chat-specific layout components

// Allow dead code for UI components that will be used in future iterations
#![allow(dead_code)]

pub mod app;
pub mod chat;
pub mod components;