//! `@vue/babel-plugin-jsx` compatibility mode selection (#3391).
//!
//! Vize's JSX/TSX compiler emits through `vize_atelier_dom` / `vize_atelier_vapor`,
//! so its output is template-compiler shaped: a block tree, lowered `v-if` /
//! `v-for`, and patch flags on every node. `@vue/babel-plugin-jsx` emits bare
//! `createVNode` calls, never opens a block, leaves `&&` / `?:` / `.map()` as
//! plain JavaScript, and by default emits **no** patch flags at all.
//!
//! Most of that difference is invisible at runtime; the rest is what this switch
//! exists for, because a project migrating off `@vue/babel-plugin-jsx` needs a
//! way to ask for babel's semantics instead of Vize's. [`JsxCompatMode::Babel`]
//! is that switch. How closely it lands is measured against the real plugin by
//! the differential oracle in `tests/babel_compat/`, whose row-by-row verdicts
//! and current totals live in `tests/BABEL_COMPAT_INVENTORY.md`.
//!
//! # Babel plugin options
//!
//! The plugin's options have no config-file spelling of their own: each is a
//! Vize option or API field on a `compile_jsx_with_babel_*` entry point, and
//! every one of them is inert unless [`JsxCompatMode::Babel`] is selected.
//!
//! | `@vue/babel-plugin-jsx` option | Vize option / API |
//! | ------------------------------ | ----------------- |
//! | `transformOn` | [`crate::BabelJsxOptions::transform_on`] |
//! | `pragma` | [`crate::compile_jsx_with_babel_pragma`] |
//! | `mergeProps` | [`crate::compile_jsx_with_babel_merge_props`] |
//! | `isCustomElement` | [`crate::BabelJsxCustomizations::is_custom_element`] |
//! | `enableObjectSlots` | [`crate::compile_jsx_with_babel_object_slots`] |
//! | any combination of the above | [`crate::compile_jsx_with_babel_customizations`] |
//!
//! `optimize` has no Vize equivalent, because Vize's output is always optimized
//! — which is what the plugin's `optimize: true` produces. `resolveType` is
//! still deferred:
//!
//! - `options/resolve_type_on` — type-driven props/emits inference needs the
//!   type-resolution work tracked on #1497 / #1502.
//!
//! # Compat under SSR output
//!
//! `@vue/babel-plugin-jsx` emits client vnode trees, so its options describe
//! client output only. SSR compilation therefore withholds the whole Babel lane
//! — the `transformOn` / `enableObjectSlots` helpers, the `isCustomElement`
//! predicate, `mergeProps: false`, and every Babel-only lowering — and falls
//! back to Vize's own SSR semantics rather than emitting a half-applied mixture.
//!
//! # Why this is not a directive prologue
//!
//! [`crate::mode::JsxOutputMode`] can be selected per component with a
//! `"use vue:vapor"` / `"use vue:vdom"` prologue, because VDOM and Vapor
//! components coexist in one module: each is an independent render function.
//! Compatibility mode is different. It changes module-level output shape (babel
//! rewrites the JSX expression in place; Vize emits a standalone `render`
//! export), so a module compiled half in compat mode and half not would emit two
//! mutually incompatible module shapes from one file. Compat is therefore a
//! **project-level** decision, configured once via `compiler.jsxCompat`, and
//! deliberately has no per-component prologue form.
//!
//! # Compat under Vapor output
//!
//! `@vue/babel-plugin-jsx` is a vdom-era plugin: every output shape it defines
//! is a `createVNode` tree, and it has no Vapor equivalent. Compat mode combined
//! with `jsxMode: "vapor"` therefore has no defined meaning and is **rejected
//! with a diagnostic** ([`unsupported_with_vapor`]) rather than silently
//! ignored. This is a deliberate answer, written down so it is not relitigated.

use crate::diagnostics::JsxDiagnostic;

/// Which JSX/TSX compilation semantics to use.
///
/// [`JsxCompatMode::Native`] is the default and must stay the default: flipping
/// it would silently change output for every existing Vize user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum JsxCompatMode {
    /// Vize's own semantics — block tree, lowered control flow, patch flags.
    #[default]
    Native,
    /// `@vue/babel-plugin-jsx` semantics, for projects migrating off the babel
    /// plugin. Opt-in via `compiler.jsxCompat: "babel"`.
    Babel,
}

impl JsxCompatMode {
    /// The canonical config value for this mode (`"native"` / `"babel"`), as
    /// used by the `compiler.jsxCompat` config key and the native bindings.
    pub const fn as_config_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Babel => "babel",
        }
    }

    /// Parse a `compiler.jsxCompat` config value (`"native"` / `"babel"`).
    ///
    /// Returns `None` for any other string so callers fall back to the default.
    /// This mirrors [`crate::mode::JsxOutputMode::from_config_str`].
    pub fn from_config_str(value: &str) -> Option<Self> {
        match value {
            "native" => Some(Self::Native),
            "babel" => Some(Self::Babel),
            _ => None,
        }
    }

    /// Whether babel-plugin-jsx semantics are requested.
    pub const fn is_babel(self) -> bool {
        matches!(self, Self::Babel)
    }
}

/// The diagnostic emitted when compat mode is combined with Vapor output.
///
/// Spans the whole component root rather than a token, because the conflict is
/// between two project-level settings and there is no single offending token in
/// the source.
pub fn unsupported_with_vapor(start: u32, end: u32) -> JsxDiagnostic {
    JsxDiagnostic::error(
        "compiler.jsxCompat: \"babel\" is not supported with Vapor output: \
         @vue/babel-plugin-jsx has no Vapor equivalent. Use jsxMode \"vdom\" for \
         babel compatibility, or drop jsxCompat to use Vize's own Vapor semantics.",
        start,
        end,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_is_the_default() {
        // Guards the compatibility promise: existing projects must not change
        // behavior when this switch is added.
        assert_eq!(JsxCompatMode::default(), JsxCompatMode::Native);
        assert!(!JsxCompatMode::default().is_babel());
    }

    #[test]
    fn config_value_round_trips() {
        for mode in [JsxCompatMode::Native, JsxCompatMode::Babel] {
            assert_eq!(
                JsxCompatMode::from_config_str(mode.as_config_str()),
                Some(mode)
            );
        }
        assert_eq!(
            JsxCompatMode::from_config_str("babel"),
            Some(JsxCompatMode::Babel)
        );
        assert_eq!(
            JsxCompatMode::from_config_str("native"),
            Some(JsxCompatMode::Native)
        );
        // Unknown values fall back rather than guessing.
        assert_eq!(JsxCompatMode::from_config_str("Babel"), None);
        assert_eq!(JsxCompatMode::from_config_str("vue"), None);
        assert_eq!(JsxCompatMode::from_config_str(""), None);
    }

    #[test]
    fn vapor_conflict_is_an_error_with_the_offending_keys_named() {
        let diagnostic = unsupported_with_vapor(3, 9);
        assert!(diagnostic.is_error());
        assert_eq!((diagnostic.start, diagnostic.end), (3, 9));
        assert_eq!(
            diagnostic.message,
            "compiler.jsxCompat: \"babel\" is not supported with Vapor output: \
             @vue/babel-plugin-jsx has no Vapor equivalent. Use jsxMode \"vdom\" for \
             babel compatibility, or drop jsxCompat to use Vize's own Vapor semantics."
        );
    }
}
