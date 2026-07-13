use vize_carton::{String, ToCompactString, cstr};
use vize_relief::CodegenOptions;

use crate::{SfcCompileOptions, SfcError};

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

pub(in crate::compile) fn finalize_output_mode(
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
