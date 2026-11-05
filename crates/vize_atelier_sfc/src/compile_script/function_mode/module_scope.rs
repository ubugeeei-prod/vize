//! Module-scope declaration emission for function mode.

use vize_carton::{String, ToCompactString};

use super::super::erasure::Erasure;
use super::super::statement_sections::extract_script_sections;

/// Split function-mode source while retaining runtime declarations only.
pub(super) fn extract(content: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    extract_script_sections(content, false).unwrap_or_else(|| {
        let setup = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.to_compact_string())
            .collect();
        (Vec::new(), setup, Vec::new())
    })
}

pub(super) fn emit_module_scope(
    output: &mut Vec<u8>,
    declarations: &[String],
    erasure: Option<&mut Erasure>,
) {
    if let Some(erasure) = erasure.filter(|_| !declarations.is_empty()) {
        let (code, _) = erasure.statements(&declarations.join("\n"), false);
        output.extend_from_slice(code.trim_end().as_bytes());
        output.push(b'\n');
        return;
    }
    for declaration in declarations {
        output.extend_from_slice(declaration.as_bytes());
        output.push(b'\n');
    }
}
