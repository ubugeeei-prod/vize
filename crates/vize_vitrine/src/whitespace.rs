//! Decode the Vue compiler's public whitespace option at FFI boundaries.
#![expect(
    clippy::disallowed_macros,
    reason = "FFI option validation needs a formatted std String error"
)]
#![expect(
    clippy::disallowed_types,
    reason = "N-API and wasm-bindgen errors use std String"
)]

use vize_atelier_core::WhitespaceStrategy;

#[derive(Clone, Copy)]
pub(crate) struct ResolvedWhitespace {
    pub strategy: WhitespaceStrategy,
    pub legacy_line_breaks: bool,
}

impl ResolvedWhitespace {
    pub fn apply<R>(self, compile: impl FnOnce() -> R) -> R {
        vize_atelier_core::parser::with_whitespace_mode(
            self.strategy,
            self.legacy_line_breaks,
            compile,
        )
    }
}

pub(crate) fn resolve_whitespace(value: Option<&str>) -> Result<ResolvedWhitespace, String> {
    match value {
        None | Some("condense") => Ok(ResolvedWhitespace {
            strategy: WhitespaceStrategy::Condense,
            legacy_line_breaks: false,
        }),
        Some("preserve") => Ok(ResolvedWhitespace {
            strategy: WhitespaceStrategy::Preserve,
            legacy_line_breaks: false,
        }),
        Some("vue2-line-breaks") => Ok(ResolvedWhitespace {
            strategy: WhitespaceStrategy::Condense,
            legacy_line_breaks: true,
        }),
        Some(value) => Err(format!(
            "Invalid whitespace `{value}`. Expected `condense`, `preserve`, or `vue2-line-breaks`."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_mode_is_explicit_and_does_not_change_vue_strategies() {
        let default = resolve_whitespace(None).unwrap();
        let preserve = resolve_whitespace(Some("preserve")).unwrap();
        let migration = resolve_whitespace(Some("vue2-line-breaks")).unwrap();
        assert_eq!(default.strategy, WhitespaceStrategy::Condense);
        assert!(!default.legacy_line_breaks);
        assert_eq!(preserve.strategy, WhitespaceStrategy::Preserve);
        assert!(!preserve.legacy_line_breaks);
        assert_eq!(migration.strategy, WhitespaceStrategy::Condense);
        assert!(migration.legacy_line_breaks);
        assert!(resolve_whitespace(Some("vue2")).is_err());
    }
}
