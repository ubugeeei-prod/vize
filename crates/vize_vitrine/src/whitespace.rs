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
    /// Vue 2.7 `preserve` text rules (#8259); implies the `Preserve` strategy.
    pub vue2_preserve: bool,
}

impl ResolvedWhitespace {
    pub fn apply<R>(self, compile: impl FnOnce() -> R) -> R {
        if self.vue2_preserve {
            return vize_atelier_core::parser::with_vue2_preserve_whitespace(compile);
        }
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
            vue2_preserve: false,
        }),
        Some("preserve") => Ok(ResolvedWhitespace {
            strategy: WhitespaceStrategy::Preserve,
            legacy_line_breaks: false,
            vue2_preserve: false,
        }),
        Some("vue2-line-breaks") => Ok(ResolvedWhitespace {
            strategy: WhitespaceStrategy::Condense,
            legacy_line_breaks: true,
            vue2_preserve: false,
        }),
        Some("vue2-preserve") => Ok(ResolvedWhitespace {
            strategy: WhitespaceStrategy::Preserve,
            legacy_line_breaks: false,
            vue2_preserve: true,
        }),
        Some(value) => Err(format!(
            "Invalid whitespace `{value}`. Expected `condense`, `preserve`, `vue2-line-breaks`, or `vue2-preserve`."
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
        assert!(!migration.vue2_preserve);
        let vue2 = resolve_whitespace(Some("vue2-preserve")).unwrap();
        assert_eq!(vue2.strategy, WhitespaceStrategy::Preserve);
        assert!(vue2.vue2_preserve);
        assert!(!vue2.legacy_line_breaks);
        assert!(resolve_whitespace(Some("vue2")).is_err());
    }
}
