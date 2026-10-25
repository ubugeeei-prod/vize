//! TypeScript pragma attribute values are literal paths, not XML entities.

use vize_carton::{FxHashSet, String};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InvalidReferencePath<'a> {
    pub(crate) path: &'a str,
    pub(crate) reason: &'static str,
}

fn delimiter(path: &str) -> Result<char, InvalidReferencePath<'_>> {
    let reason = if path.contains(['\r', '\n', '\0', '\u{2028}', '\u{2029}']) {
        Some("line breaks and NUL are not representable in a TypeScript reference path")
    } else if path.contains('"') && path.contains('\'') {
        Some("a TypeScript reference path cannot contain both quote delimiters")
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(InvalidReferencePath { path, reason });
    }
    Ok(if path.contains('"') { '\'' } else { '"' })
}

/// Production Vue projections validate before generation; the low-level legacy
/// generator still omits unrepresentable values, as it did for line breaks.
pub(crate) fn validate_reference_paths(paths: &[String]) -> Result<(), InvalidReferencePath<'_>> {
    for path in paths {
        delimiter(path)?;
    }
    Ok(())
}

pub(super) fn emit(ts: &mut String, paths: &[String]) {
    let mut seen = FxHashSet::default();
    for path in paths {
        if path.is_empty() || !seen.insert(path.as_str()) {
            continue;
        }
        let Ok(quote) = delimiter(path) else {
            continue;
        };
        ts.push_str("/// <reference path=");
        ts.push(quote);
        ts.push_str(path);
        ts.push(quote);
        ts.push_str(" />\n");
    }
}

#[cfg(test)]
mod tests;
