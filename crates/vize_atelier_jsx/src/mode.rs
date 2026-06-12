//! JSX output-mode selection.
//!
//! A JSX/TSX file compiles to either Vue VDOM or Vue Vapor output. The default
//! is chosen by configuration (#1496); individual component functions can
//! override it with a directive prologue, mirroring the `"use strict"` form:
//!
//! ```tsx
//! const Fast = () => {
//!   "use vue:vapor";
//!   return <div/>;
//! };
//! ```
//!
//! A directive that *looks* like a Vize mode directive (it starts with
//! `"use vue:"`) but does not name a known mode is reported as a diagnostic
//! rather than silently ignored, and two conflicting mode directives in one
//! function body are likewise diagnosed (see [`classify_directive`] and
//! [`crate::finder`]). Unrelated prologues such as `"use strict"` are left
//! untouched.

/// The prefix shared by every Vize JSX-mode directive (`"use vue:vapor"`,
/// `"use vue:vdom"`).
const VUE_DIRECTIVE_PREFIX: &str = "use vue:";

/// The compilation target for a JSX/TSX component.
///
/// [`JsxOutputMode::Vdom`] is the default (matching Vue's default renderer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum JsxOutputMode {
    /// Virtual DOM output (`"use vue:vdom"`).
    #[default]
    Vdom,
    /// Vapor output (`"use vue:vapor"`).
    Vapor,
}

impl JsxOutputMode {
    /// The canonical directive string that selects this mode.
    pub const fn directive(self) -> &'static str {
        match self {
            Self::Vdom => "use vue:vdom",
            Self::Vapor => "use vue:vapor",
        }
    }

    /// Resolve a component-function directive prologue string into a mode.
    ///
    /// Returns `None` for any directive that is not a Vize JSX-mode directive,
    /// so unrelated prologues (e.g. `"use strict"`) are ignored. Use
    /// [`classify_directive`] instead when a malformed `"use vue:"` directive
    /// should be diagnosed rather than ignored.
    pub fn from_directive(directive: &str) -> Option<Self> {
        match directive {
            "use vue:vapor" => Some(Self::Vapor),
            "use vue:vdom" => Some(Self::Vdom),
            _ => None,
        }
    }

    /// The canonical config value for this mode (`"vdom"` / `"vapor"`), as used
    /// by the `compiler.jsxMode` config key and the native bindings.
    pub const fn as_config_str(self) -> &'static str {
        match self {
            Self::Vdom => "vdom",
            Self::Vapor => "vapor",
        }
    }

    /// Parse a `compiler.jsxMode` config value (`"vdom"` / `"vapor"`).
    ///
    /// Returns `None` for any other string so callers can fall back to a
    /// default. This is the global-config counterpart to [`Self::from_directive`]
    /// (which parses the per-component prologue form).
    pub fn from_config_str(value: &str) -> Option<Self> {
        match value {
            "vdom" => Some(Self::Vdom),
            "vapor" => Some(Self::Vapor),
            _ => None,
        }
    }
}

/// How a single directive-prologue string relates to JSX mode selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectiveKind {
    /// A recognized mode directive (`"use vue:vapor"` / `"use vue:vdom"`).
    Mode(JsxOutputMode),
    /// A directive that opens with the Vize `"use vue:"` prefix but does not
    /// name a known mode (e.g. `"use vue:vdomm"`). The payload is the unknown
    /// suffix after the prefix, for a targeted diagnostic.
    MalformedVue,
    /// A directive unrelated to Vize JSX mode selection (e.g. `"use strict"`),
    /// left untouched.
    Unrelated,
}

/// Classify a directive-prologue string.
///
/// Distinguishes a recognized mode directive from a *malformed* one — a
/// directive that begins with `"use vue:"` but does not name a known mode,
/// which is almost always a typo (`"use vue:vdomm"`, `"use vue:VAPOR"`) and so
/// should be surfaced — and from an unrelated prologue, which is ignored.
pub fn classify_directive(directive: &str) -> DirectiveKind {
    if let Some(mode) = JsxOutputMode::from_directive(directive) {
        return DirectiveKind::Mode(mode);
    }
    if directive.starts_with(VUE_DIRECTIVE_PREFIX) {
        return DirectiveKind::MalformedVue;
    }
    DirectiveKind::Unrelated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_mode_directives() {
        assert_eq!(
            JsxOutputMode::from_directive("use vue:vapor"),
            Some(JsxOutputMode::Vapor)
        );
        assert_eq!(
            JsxOutputMode::from_directive("use vue:vdom"),
            Some(JsxOutputMode::Vdom)
        );
    }

    #[test]
    fn ignores_unrelated_directives() {
        assert_eq!(JsxOutputMode::from_directive("use strict"), None);
        assert_eq!(JsxOutputMode::from_directive("use vue:other"), None);
    }

    #[test]
    fn directive_round_trips_through_canonical_string() {
        for mode in [JsxOutputMode::Vdom, JsxOutputMode::Vapor] {
            assert_eq!(JsxOutputMode::from_directive(mode.directive()), Some(mode));
        }
    }

    #[test]
    fn config_value_round_trips() {
        for mode in [JsxOutputMode::Vdom, JsxOutputMode::Vapor] {
            assert_eq!(
                JsxOutputMode::from_config_str(mode.as_config_str()),
                Some(mode)
            );
        }
        assert_eq!(
            JsxOutputMode::from_config_str("vapor"),
            Some(JsxOutputMode::Vapor)
        );
        assert_eq!(
            JsxOutputMode::from_config_str("vdom"),
            Some(JsxOutputMode::Vdom)
        );
        assert_eq!(JsxOutputMode::from_config_str("VAPOR"), None);
        assert_eq!(JsxOutputMode::from_config_str("react"), None);
    }

    #[test]
    fn classify_recognizes_known_modes() {
        assert_eq!(
            classify_directive("use vue:vapor"),
            DirectiveKind::Mode(JsxOutputMode::Vapor)
        );
        assert_eq!(
            classify_directive("use vue:vdom"),
            DirectiveKind::Mode(JsxOutputMode::Vdom)
        );
    }

    #[test]
    fn classify_flags_malformed_vue_directives() {
        // Typos in the suffix are the common case we want to catch.
        assert_eq!(
            classify_directive("use vue:vdomm"),
            DirectiveKind::MalformedVue
        );
        assert_eq!(
            classify_directive("use vue:VAPOR"),
            DirectiveKind::MalformedVue
        );
        assert_eq!(classify_directive("use vue:"), DirectiveKind::MalformedVue);
    }

    #[test]
    fn classify_ignores_unrelated_prologues() {
        assert_eq!(classify_directive("use strict"), DirectiveKind::Unrelated);
        assert_eq!(classify_directive("use asm"), DirectiveKind::Unrelated);
        // No `vue:` prefix, so not ours even though it mentions vue.
        assert_eq!(classify_directive("use vuex"), DirectiveKind::Unrelated);
    }
}
