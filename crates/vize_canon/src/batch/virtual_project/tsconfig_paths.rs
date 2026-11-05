//! Canon path/JSONC facade; config input custody stays with CompilerInputs.
use super::tsconfig_gen::{compiler_inputs::CompilerInputs, compiler_options::CanonConfigHost};
use crate::batch::error::CorsaResult;
use serde_json::Value;
pub(super) use shared::normalize_path_lexically;
#[cfg(test)]
pub(super) use shared::strip_json_comments;
use std::path::{Path, PathBuf};
use vize_carton::typescript_project::{self as shared, CompilerConfigHost};

pub(crate) fn resolve_extended_tsconfig_path(path: &Path, extends: &str) -> Option<PathBuf> {
    resolve_extended_tsconfig_path_with_inputs(path, extends, None)
}
pub(super) fn resolve_extended_tsconfig_path_with_inputs(
    path: &Path,
    extends: &str,
    inputs: Option<&CompilerInputs>,
) -> Option<PathBuf> {
    CanonConfigHost { inputs }.resolve_extends(path, extends)
}
pub(super) fn resolve_extended_tsconfig_path_with_dependencies(
    path: &Path,
    extends: &str,
    dependencies: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
    shared::resolve_extended_tsconfig_path_with_dependencies(
        path,
        extends,
        &CanonConfigHost { inputs: None },
        dependencies,
    )
}
pub(crate) fn parse_jsonc_value(content: &str) -> CorsaResult<Value> {
    Ok(shared::parse_jsonc_value(content)?)
}

#[cfg(test)]
mod bom_tests;
