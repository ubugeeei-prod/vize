//! Guard scaffolding inputs without interpreting or inventing their types.

use core::fmt;
use vize_extension_contract::typed_expression::{TypedBinding, TypedExpressionBatch};
use vize_l0::{String, cstr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentError(pub String);

impl fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "typed MoonBit environment refused: {}", self.0)
    }
}

pub(super) fn batch(batch: &TypedExpressionBatch) -> Result<(), EnvironmentError> {
    bindings(&batch.environment)?;
    let mut ids = std::collections::BTreeSet::new();
    let mut bytes = 1024usize;
    for expression in &batch.expressions {
        if !ids.insert(expression.id) {
            return Err(EnvironmentError(cstr!(
                "duplicate expression id {}",
                expression.id
            )));
        }
        if expression.span.end.checked_sub(expression.span.start)
            != u32::try_from(expression.source.len()).ok()
        {
            return Err(EnvironmentError(cstr!(
                "expression {} does not have an exact authored byte span",
                expression.id
            )));
        }
        bindings(&expression.locals)?;
        let scope = batch.scope(expression);
        bytes = bytes
            .saturating_add(256)
            .saturating_add(expression.source.len());
        for binding in scope {
            bytes = bytes
                .saturating_add(binding.signature.len().saturating_mul(2))
                .saturating_add(binding.name.len().saturating_mul(3))
                .saturating_add(16);
        }
        if bytes > 128 * 1024 - 1 {
            return Err(EnvironmentError(cstr!(
                "typed projection exceeds the 128 KiB transport budget"
            )));
        }
    }
    Ok(())
}

fn bindings(bindings: &[TypedBinding]) -> Result<(), EnvironmentError> {
    let mut names = std::collections::BTreeSet::new();
    for binding in bindings {
        let name = binding.name.as_str();
        if !name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            || name.starts_with("__vize_")
        {
            return Err(EnvironmentError(cstr!("unsupported binding name {name:?}")));
        }
        if !names.insert(name) {
            return Err(EnvironmentError(cstr!("duplicate binding {name:?}")));
        }
        if !signature(&binding.signature) {
            return Err(EnvironmentError(cstr!(
                "binding {name:?} has a missing or unsupported signature"
            )));
        }
    }
    Ok(())
}

/// Only a single balanced type expression can enter a parameter. `moonc`
/// owns its grammar and meaning. Imported types require an import producer
/// and are refused in this version of the environment contract.
fn signature(source: &str) -> bool {
    if source.trim().is_empty() {
        return false;
    }
    let mut stack = Vec::new();
    for character in source.chars() {
        match character {
            '(' | '[' => stack.push(character),
            ')' if stack.pop() == Some('(') => {}
            ']' if stack.pop() == Some('[') => {}
            ',' if !stack.is_empty() => {}
            ' ' | '?' | '-' | '>' => {}
            c if c.is_ascii_alphanumeric() || c == '_' => {}
            _ => return false,
        }
    }
    stack.is_empty()
}
