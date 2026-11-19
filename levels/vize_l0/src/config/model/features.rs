//! Auxiliary feature flags derived from shared configuration.

use super::{JsxCompat, JsxMode, VueVersion};

/// Feature flags parsed from config keys outside stable Rust model fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigFeatureFlags {
    /// Resolve Vue 3 Options API template bindings during type checking.
    /// Default-on (matches vue-tsc): an Options API SFC's template bindings
    /// (`data`/`computed`/`methods`/`props`) resolve without configuration.
    /// Set `typeChecker.optionsApi: false` to opt out. Available in the standard
    /// build (not a legacy feature).
    pub type_checker_options_api: bool,
    pub type_checker_legacy_vue2: bool,
    /// Opt-in type-checking of `.jsx`/`.tsx` Vue components (#1497). Default-off
    /// so mixed Vue/React repositories do not accidentally route React `.tsx`
    /// through the Vue JSX checker. Set `typeChecker.jsxTypecheck: true` or
    /// opt into `experimentals.jsxVapor` to route `.jsx`/`.tsx` through the
    /// Vize JSX virtual-TS path instead of the verbatim passthrough.
    pub type_checker_jsx_typecheck: bool,
    pub language_server_legacy_vue2: Option<bool>,
    /// Dialect selected by `vue.version`; `None` when the key is absent
    /// (modern Vue 3). Validated at parse time — unknown or ambiguous values
    /// fail config loading instead of silently picking a line. Groundwork for
    /// legacy Vue support (#1392): consumers thread this into parser and
    /// transform options in follow-ups.
    pub vue_version: Option<VueVersion>,
    /// Default JSX/TSX output backend selected by `compiler.jsxMode` (#1496);
    /// `None` when the key is absent (treated as VDOM). The JS plugins and the
    /// native `compileJsx` binding thread this into the per-component
    /// mode-selection logic so a single module can still mix VDOM and Vapor via
    /// `"use vue:*"` directives.
    pub jsx_mode: Option<JsxMode>,
    /// JSX/TSX compatibility semantics selected by `compiler.jsxCompat` (#3391);
    /// `None` when the key is absent (treated as `native`). Opting into `babel`
    /// asks the JSX compiler for `@vue/babel-plugin-jsx` semantics instead of
    /// Vize's own; the JS plugins and the native `compileJsx` binding thread it
    /// through the same way as `jsx_mode`.
    pub jsx_compat: Option<JsxCompat>,
    pub experimental_vapor: bool,
    pub experimental_jsx_vapor: bool,
    pub experimental_in_tag_comments: bool,
    pub experimental_patterned_template: bool,
    pub experimental_server_script: bool,
}

impl Default for ConfigFeatureFlags {
    fn default() -> Self {
        Self {
            // Options API resolution is default-on (matches vue-tsc).
            type_checker_options_api: true,
            type_checker_legacy_vue2: false,
            type_checker_jsx_typecheck: false,
            language_server_legacy_vue2: None,
            vue_version: None,
            jsx_mode: None,
            jsx_compat: None,
            experimental_vapor: false,
            experimental_jsx_vapor: false,
            experimental_in_tag_comments: false,
            experimental_patterned_template: false,
            experimental_server_script: false,
        }
    }
}
