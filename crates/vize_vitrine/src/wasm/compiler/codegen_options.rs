use crate::CompilerOptions;
use vize_atelier_core::options::{CodegenExperimentalOptions, CodegenOptions};

pub(in crate::wasm) fn compiler_codegen_options(
    opts: &CompilerOptions,
    default_filename: &str,
) -> CodegenOptions {
    let mut codegen_options = CodegenOptions {
        filename: opts.filename.as_deref().unwrap_or(default_filename).into(),
        component_name: self_component_name(opts).map(Into::into),
        ..CodegenOptions::default()
    };
    if let Some(runtime_module_name) = opts.runtime_module_name.as_deref() {
        codegen_options.runtime_module_name = runtime_module_name.into();
    }
    if let Some(runtime_global_name) = opts.runtime_global_name.as_deref() {
        codegen_options.runtime_global_name = runtime_global_name.into();
    }
    codegen_options
}

pub(in crate::wasm) fn compiler_codegen_experimental_options(
    opts: &CompilerOptions,
) -> CodegenExperimentalOptions {
    CodegenExperimentalOptions {
        component_name: None,
        self_component: opts.experimental_self_component.unwrap_or(false),
    }
}

pub(in crate::wasm) fn self_component_name(opts: &CompilerOptions) -> Option<String> {
    opts.component_name
        .clone()
        .or_else(|| component_name_from_filename(opts.filename.as_deref()))
}

fn component_name_from_filename(filename: Option<&str>) -> Option<String> {
    let filename = filename?;
    let stem = std::path::Path::new(filename).file_stem()?.to_str()?.trim();
    (!stem.is_empty()).then(|| stem.to_string())
}
