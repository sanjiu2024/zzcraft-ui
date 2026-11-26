//! Pieces that GPUI Kit does not ship, factored out so more than one application
//! can use them.
//!
//! The modules are independent on purpose:
//!
//! - [`format`] is pure data formatting (money, counts, compact notation) and
//!   works without any UI dependency.
//! - [`table`] and [`form`] build on GPUI Kit's primitives and are only compiled
//!   with the `gpui` feature.
//!
//! ```
//! use zzcraft_ui::format::{Currency, format_money};
//!
//! assert_eq!(format_money(1234.5, Currency::USD), "$1,234.50");
//! ```

pub mod form;
pub mod format;
pub mod table;
