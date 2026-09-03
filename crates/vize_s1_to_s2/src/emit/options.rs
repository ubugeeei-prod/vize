//! Emit-time options for the S2 DOM lane (P2-11).
//!
//! The shipped DOM codegen reads its emission-only settings from
//! `CodegenOptions`. The S2 emitter mirrors the subset it honours here so
//! the atelier_dom dual-run can pin each option against the shipped lane
//! one at a time. A field missing from this struct is not a default the
//! emitter silently assumes — it is production surface the series has
//! not reached yet, and the witness for it does not exist.

/// Which module form the render function is emitted in — the shipped
/// lane's `CodegenMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DomEmitMode {
    /// `const { … } = Vue` destructure plus the full
    /// `function render(_ctx, _cache, $props, $setup, $data, $options)`
    /// signature (the shipped default).
    #[default]
    Function,
    /// `import { … } from "vue"` plus `export function render(_ctx, _cache)`.
    /// The six-argument signature returns with binding metadata, which the
    /// emitter does not carry yet.
    Module,
}

impl DomEmitMode {
    /// The render-function header the shipped lane writes for this mode
    /// without binding metadata.
    #[must_use]
    pub(super) const fn render_signature(self) -> &'static str {
        match self {
            Self::Function => "function render(_ctx, _cache, $props, $setup, $data, $options) {",
            Self::Module => "export function render(_ctx, _cache) {",
        }
    }
}

/// Emission settings the S2 DOM emitter honours.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomEmitOptions<'a> {
    /// Module or function output.
    pub mode: DomEmitMode,
    /// The module the helper imports name in [`DomEmitMode::Module`]
    /// (`"vue"` by default).
    pub runtime_module_name: &'a str,
    /// The global the helper destructure reads in
    /// [`DomEmitMode::Function`] (`"Vue"` by default).
    pub runtime_global_name: &'a str,
}

impl DomEmitOptions<'static> {
    /// The shipped lane's `CodegenOptions::default()` projected onto the
    /// fields the emitter honours.
    pub const DEFAULT: Self = Self {
        mode: DomEmitMode::Function,
        runtime_module_name: "vue",
        runtime_global_name: "Vue",
    };
}

impl Default for DomEmitOptions<'static> {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::{DomEmitMode, DomEmitOptions};

    #[test]
    fn default_options_project_the_shipped_codegen_defaults() {
        assert_eq!(
            DomEmitOptions::default(),
            DomEmitOptions {
                mode: DomEmitMode::Function,
                runtime_module_name: "vue",
                runtime_global_name: "Vue",
            }
        );
    }

    #[test]
    fn render_signatures_match_the_shipped_lane_per_mode() {
        assert_eq!(
            DomEmitMode::Function.render_signature(),
            "function render(_ctx, _cache, $props, $setup, $data, $options) {"
        );
        assert_eq!(
            DomEmitMode::Module.render_signature(),
            "export function render(_ctx, _cache) {"
        );
    }
}
