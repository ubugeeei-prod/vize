//! Scope identity for original scoped styles, independent of legacy helpers.

use super::NativeSfcCompileError;
use std::hash::{Hash, Hasher};
use vize_l0::String;
use vize_l4::module::ScopeId;

pub(crate) fn for_styles(
    descriptor: StyleBlocks<'_, '_>,
    explicit: Option<&str>,
    filename: &str,
) -> Result<Option<String>, NativeSfcCompileError> {
    if !descriptor
        .styles()
        .any(|style| style.attrs().iter().any(|attr| attr.name == "scoped"))
    {
        return Ok(None);
    }
    if let Some(value) = explicit {
        ScopeId::new(value).map_err(NativeSfcCompileError::Assembly)?;
        return Ok(Some(value.into()));
    }
    // The established ordinary SFC filename policy: DefaultHasher, low 32 bits,
    // eight lowercase hex digits. Source-bound dev-only oracle/runtime laws
    // independently compare this complete identity against ordinary compilation.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    filename.hash(&mut hasher);
    let value = hasher.finish() & 0xffff_ffff;
    let mut output = String::from("data-v-");
    for shift in (0..32).step_by(4).rev() {
        output.push(char::from_digit(((value >> shift) & 0xf) as u32, 16).unwrap_or('0'));
    }
    Ok(Some(output))
}

// Whole components and independently checked styles use the same scope policy.
// Only the opt-in component entry point needs to narrow its descriptor.
#[cfg(not(feature = "native-scoped-scss"))]
use vize_l1::container::vue::AdmittedDescriptor as StyleBlocks;
#[cfg(feature = "native-scoped-scss")]
use vize_l1::container::vue::CheckedStyleBlocks as StyleBlocks;

#[cfg(not(feature = "native-scoped-scss"))]
pub(crate) use for_styles as for_component;

#[cfg(feature = "native-scoped-scss")]
pub(crate) fn for_component(
    descriptor: vize_l1::container::vue::AdmittedDescriptor<'_, '_>,
    explicit: Option<&str>,
    filename: &str,
) -> Result<Option<String>, NativeSfcCompileError> {
    for_styles(descriptor.into(), explicit, filename)
}
