//! Keep upstream CSS parser defects within the existing invalid-CSS fallback.

use std::panic::{AssertUnwindSafe, catch_unwind};

use lightningcss::error::{Error, ParserError};
use lightningcss::stylesheet::{ParserOptions, StyleSheet};

pub(crate) fn parse_stylesheet(source: &str) -> Result<StyleSheet<'_>, Error<ParserError<'_>>> {
    // Past this depth the engine only reports a parse error, after
    // exponential work; report invalid CSS without it.
    if vize_l1::css::nesting::css_nesting_exceeds_max_depth(source) {
        return Err(Error {
            kind: ParserError::InvalidValue,
            loc: None,
        });
    }
    catch_unwind(AssertUnwindSafe(|| {
        StyleSheet::parse(source, ParserOptions::default())
    }))
    .unwrap_or_else(|_| {
        Err(Error {
            kind: ParserError::InvalidValue,
            loc: None,
        })
    })
}
