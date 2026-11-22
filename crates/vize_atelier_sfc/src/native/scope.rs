//! Scope identity for original scoped styles, independent of legacy helpers.

use super::NativeSfcCompileError;
use std::hash::{Hash, Hasher};
use vize_l0::String;

pub(crate) fn for_styles<'o, 'a: 'o>(
    #[cfg(not(feature = "native-scoped-scss"))]
    descriptor: vize_l1::container::vue::AdmittedDescriptor<'o, 'a>,
    #[cfg(feature = "native-scoped-scss")] descriptor: impl Into<
        vize_l1::container::vue::CheckedStyleBlocks<'o, 'a>,
    >,
    explicit: Option<&str>,
    filename: &str,
) -> Result<Option<String>, NativeSfcCompileError> {
    #[cfg(feature = "native-scoped-scss")]
    let descriptor = descriptor.into();
    if !descriptor
        .styles()
        .any(|style| style.attrs().iter().any(|attr| attr.name == "scoped"))
    {
        return Ok(None);
    }
    if let Some(value) = explicit {
        vize_l4::module::ScopeId::new(value).map_err(NativeSfcCompileError::Assembly)?;
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
