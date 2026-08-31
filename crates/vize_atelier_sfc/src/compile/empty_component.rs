use crate::types::{SfcCompileOptions, SfcCompileResult, SfcError, SfcMacroArtifact};
use vize_atelier_core::CodegenOptions;
use vize_s0::String;

use super::output_module::{append_css_modules_assignment, finalize_output_mode};
use super::styles::CompiledStyles;
use super::trim_trailing_newlines;

pub(super) struct EmptyComponentContext<'a> {
    pub(super) is_vapor: bool,
    pub(super) compiled_styles: &'a CompiledStyles,
    pub(super) css: Option<String>,
    pub(super) errors: Vec<SfcError>,
    pub(super) warnings: Vec<SfcError>,
    pub(super) macro_artifacts: Vec<SfcMacroArtifact>,
    pub(super) options: &'a SfcCompileOptions,
    pub(super) codegen_options: &'a CodegenOptions,
}

pub(super) fn compile_empty_component(context: EmptyComponentContext<'_>) -> SfcCompileResult {
    let EmptyComponentContext {
        is_vapor,
        compiled_styles,
        css,
        errors,
        mut warnings,
        macro_artifacts,
        options,
        codegen_options,
    } = context;
    // Nuxt/Vue projects can contain empty placeholder SFCs; the host compiler
    // treats them as importable empty components, so Vize must not fail builds.
    let mut code = String::from("const _sfc_main = ");
    if is_vapor {
        code.push_str("{ __vapor: true }");
    } else {
        code.push_str("{}");
    }
    if !compiled_styles.css_modules.is_empty() {
        code.push('\n');
        append_css_modules_assignment(&mut code, "_sfc_main", &compiled_styles.css_modules);
    }
    code.push_str("\nexport default _sfc_main\n");

    finalize_output_mode(&mut code, &mut warnings, options, codegen_options);
    trim_trailing_newlines(&mut code);

    SfcCompileResult {
        code,
        css,
        map: None,
        errors,
        warnings,
        bindings: None,
        macro_artifacts,
    }
}
