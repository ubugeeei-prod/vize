//! Assemble the exact source ESM validator into the private native Node module.

const VALIDATION_IMPORT: &str =
    "import { createVueOptionValidator } from \"./vite-runtime-validation.js\";\n";
const VALIDATION: &str = include_str!("../vite-runtime-validation.js");

pub(super) fn vite_runtime() -> vize_l0::String {
    let (before, after) = super::VITE_RUNTIME
        .split_once(VALIDATION_IMPORT)
        .expect("one owned validator import in the product runtime");
    assert_eq!(after.matches(VALIDATION_IMPORT).count(), 0);
    let mut source =
        vize_l0::String::with_capacity(VALIDATION.len() + 1 + before.len() + after.len());
    source.push_str(VALIDATION);
    source.push('\n');
    source.push_str(before);
    source.push_str(after);
    source
}

#[cfg(test)]
mod tests;
