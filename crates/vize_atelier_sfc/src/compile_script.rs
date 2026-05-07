//! Script compilation for Vue SFCs.
//!
//! This module handles compilation of `<script>` and `<script setup>` blocks,
//! following the Vue.js core output format.

pub(crate) mod artifacts;
pub mod function_mode;
pub mod import_utils;
pub mod inline;
pub(crate) mod lazy_hydration;
pub mod macros;
pub mod props;
pub(crate) mod runtime_bindings;
pub mod statement_sections;
#[cfg(test)]
mod tests;
pub mod typescript;

use crate::types::{BindingMetadata, ScriptCompileOptions, SfcDescriptor, SfcError};

use self::function_mode::compile_script_setup;
use self::lazy_hydration::transform_lazy_hydration_macros;
use self::typescript::transform_typescript_to_js;

// Re-export commonly used items
pub use self::function_mode::compile_script_setup as compile_script_setup_function_mode;
pub use self::import_utils::{extract_import_identifiers, process_import_for_types};
pub use self::inline::compile_script_setup_inline;
pub(crate) use self::inline::compile_script_setup_inline_with_context;
pub use self::macros::{
    is_macro_call_line, is_multiline_macro_start, is_paren_macro_start, is_props_destructure_line,
};
pub use self::props::{
    extract_emit_names_from_type, extract_prop_types_from_type, extract_with_defaults_defaults,
    is_valid_identifier, PropTypeInfo,
};
use vize_carton::{String, ToCompactString};

/// Script compilation result
pub struct ScriptCompileResult {
    pub code: String,
    pub bindings: Option<BindingMetadata>,
}

/// Template parts for inline compilation
pub struct TemplateParts<'a> {
    pub imports: &'a str,
    pub hoisted: &'a str,
    pub render_fn: &'a str,
    pub render_fn_name: &'a str,
    /// Component/directive resolution statements (inside render function, before return)
    pub preamble: &'a str,
    pub render_body: &'a str,
    /// Whether render_body contains a full statement block instead of a return expression
    pub render_is_block: bool,
}

/// Compile script block(s)
#[allow(dead_code)]
pub fn compile_script(
    descriptor: &SfcDescriptor,
    _options: &ScriptCompileOptions,
    component_name: &str,
    is_vapor: bool,
    is_ts: bool,
) -> Result<ScriptCompileResult, SfcError> {
    // Handle script setup
    if let Some(script_setup) = &descriptor.script_setup {
        let template_content = descriptor.template.as_ref().map(|t| t.content.as_ref());
        let transformed = transform_lazy_hydration_macros(&script_setup.content);
        let script_content = transformed
            .as_ref()
            .map(|result| result.code.as_str())
            .unwrap_or(&script_setup.content);
        let mut result = compile_script_setup(
            script_content,
            component_name,
            is_vapor,
            is_ts,
            template_content,
        )?;
        if let Some(transformed) = transformed {
            let mut code = transformed.preamble;
            code.push_str(&result.code);
            result.code = code;
        }
        Ok(result)
    } else if let Some(script) = &descriptor.script {
        // Use regular script, wrapped in __sfc__
        let mut code = String::default();
        code.push_str(&script.content);
        if is_vapor {
            code.push_str("\nconst __sfc__ = { ...(__default__ || {}), __vapor: true }\n");
        } else {
            code.push_str("\nconst __sfc__ = __default__\n");
        }
        // Transform TypeScript to JavaScript only when output is not TS.
        let final_code = if is_ts {
            code
        } else {
            transform_typescript_to_js(&code)
        };
        Ok(ScriptCompileResult {
            code: final_code,
            bindings: None,
        })
    } else {
        // No script - generate empty component
        if is_vapor {
            Ok(ScriptCompileResult {
                code: "const __sfc__ = { __vapor: true }\n".to_compact_string(),
                bindings: None,
            })
        } else {
            Ok(ScriptCompileResult {
                code: "const __sfc__ = {}\n".to_compact_string(),
                bindings: None,
            })
        }
    }
}
