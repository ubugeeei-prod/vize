//! One transport for the public SFC runtime names on every native entry point.

use vize_atelier_core::CodegenOptions;

pub(super) fn sfc_codegen_options(
    source_map: bool,
    runtime_module_name: Option<&str>,
    runtime_global_name: Option<&str>,
) -> CodegenOptions {
    let defaults = CodegenOptions::default();
    CodegenOptions {
        source_map,
        runtime_module_name: runtime_module_name.map_or(defaults.runtime_module_name, Into::into),
        runtime_global_name: runtime_global_name.map_or(defaults.runtime_global_name, Into::into),
        ..defaults
    }
}
