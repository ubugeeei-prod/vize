//! Keep upstream CSS parser defects within the existing invalid-CSS fallback.

use std::panic::{AssertUnwindSafe, catch_unwind};

use lightningcss::error::{Error, ParserError};
use lightningcss::stylesheet::{ParserOptions, StyleSheet};

pub(crate) fn parse_stylesheet(source: &str) -> Result<StyleSheet<'_>, Error<ParserError<'_>>> {
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
