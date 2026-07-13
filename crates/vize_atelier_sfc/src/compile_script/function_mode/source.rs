//! Source-aware setup for function-mode script compilation.

use crate::script::ScriptCompileContext;
use crate::types::SfcError;

use super::super::ScriptCompileResult;
use super::compiler::compile_script_setup_from_source;

/// Compile script setup using its legacy single-source API.
#[allow(dead_code)]
pub fn compile_script_setup(
    content: &str,
    component_name: &str,
    is_vapor: bool,
    is_ts: bool,
    template_content: Option<&str>,
) -> Result<ScriptCompileResult, SfcError> {
    compile_script_setup_from_source(
        content,
        component_name,
        is_vapor,
        is_ts,
        is_ts,
        template_content,
        None,
        None,
    )
}

pub(super) fn build_context(
    content: &str,
    normal_script_content: Option<&str>,
    filename: Option<&str>,
    source_is_ts: bool,
) -> ScriptCompileContext {
    let mut context = ScriptCompileContext::new(content);
    if let Some(normal) = normal_script_content.filter(|normal| !normal.is_empty()) {
        context.collect_types_from(normal);
    }
    if let Some(filename) = filename.filter(|filename| !filename.is_empty()) {
        context.collect_imported_types_from_path(content, filename, source_is_ts);
        if let Some(normal) = normal_script_content.filter(|normal| !normal.is_empty()) {
            context.collect_imported_types_from_path(normal, filename, source_is_ts);
        }
    }
    context
}
