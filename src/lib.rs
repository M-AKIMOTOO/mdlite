//! Lightweight Markdown rendering utilities for terminal output.
//!
//! `mdlite` can be used as a library when you want to render a subset of
//! Markdown into plain text or ANSI-colored terminal text.
//!
//! # Examples
//!
//! ```rust
//! let rendered = mdlite::render_markdown("# Hello\n\n- one\n- two\n", false);
//! assert!(rendered.contains("Hello"));
//! assert!(rendered.contains("- one"));
//! ```
//!
//! ```no_run
//! let rendered = mdlite::render_markdown("# Hello from mdlite\n", true);
//! mdlite::run_pager(&rendered).unwrap();
//! ```

pub mod markdown;
pub mod pager;

/// Renders a lightweight subset of Markdown into terminal-friendly text.
pub use markdown::render_markdown;
/// Displays rendered text through the built-in pager.
pub use pager::run_pager;
