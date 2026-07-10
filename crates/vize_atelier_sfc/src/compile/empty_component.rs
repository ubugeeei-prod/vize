use crate::types::{SfcCompileOptions, SfcCompileResult, SfcError, SfcMacroArtifact};
use vize_carton::String;

use super::output_module::append_css_modules_assignment;
use super::styles::CompiledStyles;
use super::{finalize_output_mode, trim_trailing_newlines};

pub(super) fn compile_empty_component(
    is_vapor: bool,
    compiled_styles: &CompiledStyles,
    css: Option<String>,
    errors: Vec<SfcError>,
    mut warnings: Vec<SfcError>,
    macro_artifacts: Vec<SfcMacroArtifact>,
    options: &SfcCompileOptions,
) -> SfcCompileResult {
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

    finalize_output_mode(&mut code, &mut warnings, options);
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
