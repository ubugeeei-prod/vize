//! Shared SFC render output assembly.

use crate::types::{CssModuleMapping, SfcCompileOptions, SfcError, css_modules_object_literal};
use vize_atelier_core::CodegenOptions;
use vize_carton::{String, ToCompactString, cstr};

/// The render function a generated SFC component should expose.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RenderFunctionName {
    Render,
    SfcRender,
    SsrRender,
}

impl RenderFunctionName {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Render => "render",
            Self::SfcRender => "_sfc_render",
            Self::SsrRender => "ssrRender",
        }
    }

    fn component_field(self) -> &'static str {
        match self {
            Self::Render | Self::SfcRender => "render",
            Self::SsrRender => "ssrRender",
        }
    }
}

/// Structural output module used before SFC code is flattened into a string.
///
/// This is intentionally small: it gives the SFC layer a typed boundary for
/// imports/hoists/functions/exports without changing backend emitters yet.
#[derive(Debug, Default)]
pub(crate) struct OutputModule {
    pub(crate) imports: String,
    pub(crate) hoists: String,
    pub(crate) functions: String,
    pub(crate) exports: String,
}

impl OutputModule {
    pub(crate) fn from_render_chunks(imports: String, functions: String) -> Self {
        Self {
            imports,
            functions,
            ..Self::default()
        }
    }

    pub(crate) fn function_base_offset(&self) -> usize {
        self.imports.len() + self.hoists.len() + 1
    }

    pub(crate) fn into_code(self) -> String {
        let mut code = String::default();
        code.push_str(&self.imports);
        code.push_str(&self.hoists);
        code.push('\n');
        code.push_str(&self.functions);
        code.push('\n');
        code.push_str(&self.exports);
        code
    }
}

fn create_standalone_import_warning() -> SfcError {
    SfcError {
        message: "Standalone SFC output still contains non-Vue ES module imports; CDN evaluation requires those dependencies to be provided separately."
            .to_compact_string(),
        code: Some("STANDALONE_EXTERNAL_IMPORT".to_compact_string()),
        loc: None,
    }
}

fn rewrite_runtime_import_line(
    trimmed: &str,
    runtime_module_name: &str,
    runtime_global_name: &str,
) -> Option<String> {
    let rest = trimmed.strip_prefix("import {")?;
    let (specifiers, rest) = rest.split_once("} from ")?;
    let source = rest.trim().trim_end_matches(';');
    let expected_double = cstr!("\"{runtime_module_name}\"");
    let expected_single = cstr!("'{runtime_module_name}'");
    if source != expected_double && source != expected_single {
        return None;
    }

    let bindings: Vec<_> = specifiers
        .split(',')
        .filter_map(|specifier| {
            let specifier = specifier.trim();
            let specifier = specifier.strip_prefix("type ").unwrap_or(specifier).trim();
            if specifier.is_empty() {
                return None;
            }

            if let Some((imported, local)) = specifier.split_once(" as ") {
                Some(cstr!("{}: {}", imported.trim(), local.trim()))
            } else {
                Some(specifier.to_compact_string())
            }
        })
        .collect();

    if bindings.is_empty() {
        return Some(String::default());
    }

    let mut joined = String::default();
    for (index, binding) in bindings.iter().enumerate() {
        if index > 0 {
            joined.push_str(", ");
        }
        joined.push_str(binding);
    }

    Some(cstr!("const {{ {} }} = {}", joined, runtime_global_name))
}

fn rewrite_module_sfc_to_standalone(
    code: &str,
    runtime_module_name: &str,
    runtime_global_name: &str,
) -> (String, bool) {
    let mut output = String::with_capacity(code.len());
    let mut has_external_imports = false;

    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            if let Some(rewritten) =
                rewrite_runtime_import_line(trimmed, runtime_module_name, runtime_global_name)
            {
                if !rewritten.is_empty() {
                    output.push_str(&rewritten);
                    output.push('\n');
                }
                continue;
            }
            has_external_imports = true;
        }

        let mut rewritten = line
            .replace("export function render(", "function render(")
            .replace("export function ssrRender(", "function ssrRender(");
        if let Some(index) = rewritten.find("export default")
            && rewritten[..index].trim().is_empty()
        {
            rewritten.replace_range(index..index + "export default".len(), "return");
        }
        output.push_str(&rewritten);
        output.push('\n');
    }

    (output, has_external_imports)
}

pub(super) fn finalize_output_mode(
    code: &mut String,
    warnings: &mut Vec<SfcError>,
    options: &SfcCompileOptions,
    codegen_options: &CodegenOptions,
) {
    if !options.script.inline_template {
        return;
    }

    let (rewritten, has_external_imports) = rewrite_module_sfc_to_standalone(
        code,
        codegen_options.runtime_module_name.as_str(),
        codegen_options.runtime_global_name.as_str(),
    );
    *code = rewritten;

    if has_external_imports {
        warnings.push(create_standalone_import_warning());
    }
}

pub(crate) fn rewrite_client_render_for_sfc_main(template_code: &str) -> String {
    if template_code.contains("export function render(") {
        return template_code
            .replacen("export function render(", "function _sfc_render(", 1)
            .to_compact_string();
    }

    if template_code.contains("function render(") {
        return template_code
            .replacen("function render(", "function _sfc_render(", 1)
            .to_compact_string();
    }

    template_code.to_compact_string()
}

pub(crate) fn append_css_modules_assignment(
    code: &mut String,
    target: &str,
    css_modules: &[CssModuleMapping],
) {
    if css_modules.is_empty() {
        return;
    }

    code.push_str(target);
    code.push_str(".__cssModules = ");
    code.push_str(&css_modules_object_literal(css_modules, ""));
    code.push('\n');
}

pub(crate) fn append_component_render_export(
    code: &mut String,
    target: &str,
    render: RenderFunctionName,
    css_modules: &[CssModuleMapping],
) {
    code.push_str(target);
    code.push('.');
    code.push_str(render.component_field());
    code.push_str(" = ");
    code.push_str(render.as_str());
    code.push('\n');
    append_css_modules_assignment(code, target, css_modules);
    code.push_str("export default ");
    code.push_str(target);
    code.push('\n');
}
