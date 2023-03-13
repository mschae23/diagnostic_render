//! This crate provides an [ASCII renderer] for printing formatted [diagnostics]
//! like error messages and warnings on some source code.
//!
//! These diagnostics contain annotations that are shown directly on the lines
//! in the source they refer to, as well as notes shown after the source.
//!
//! # Example
//! ```
//! // TODO give an example here
//! ```
//!
//! [ASCII renderer]: render::DiagnosticRenderer
//! [diagnostics]: diagnostic::Diagnostic

pub mod file;
pub mod diagnostic;
pub mod render;
