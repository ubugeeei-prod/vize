//! Keep upstream CSS parser defects within the existing invalid-CSS fallback.

use std::panic::{AssertUnwindSafe, catch_unwind};

use lightningcss::stylesheet::{ParserOptions, StyleSheet};

pub(crate) fn parse_stylesheet(source: &str) -> Option<StyleSheet<'_>> {
    catch_unwind(AssertUnwindSafe(|| {
        StyleSheet::parse(source, ParserOptions::default())
    }))
    .ok()?
    .ok()
}
