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
    /// Resolve a component-function directive prologue string into a mode.
    ///
    /// Returns `None` for any directive that is not a Vize JSX-mode directive,
    /// so unrelated prologues (e.g. `"use strict"`) are ignored.
    pub fn from_directive(directive: &str) -> Option<Self> {
        match directive {
            "use vue:vapor" => Some(Self::Vapor),
            "use vue:vdom" => Some(Self::Vdom),
            _ => None,
        }
    }
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
}
