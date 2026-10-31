//! Assemble the exact source ESM validator into the private native Node module.

const VALIDATION_IMPORT: &str =
    "import { createVueOptionValidator } from \"./vite-runtime-validation.js\";\n";
const VALIDATION: &str = include_str!("../vite-runtime-validation.js");

pub(super) fn vite_runtime() -> Result<vize_l0::String, &'static str> {
    assemble(super::VITE_RUNTIME)
}

fn assemble(runtime: &str) -> Result<vize_l0::String, &'static str> {
    let Some((before, after)) = runtime.split_once(VALIDATION_IMPORT) else {
        return Err("missing owned validator import in the product runtime");
    };
    if after.contains(VALIDATION_IMPORT) {
        return Err("duplicate owned validator import in the product runtime");
    }
    let mut source =
        vize_l0::String::with_capacity(VALIDATION.len() + 1 + before.len() + after.len());
    source.push_str(VALIDATION);
    source.push('\n');
    source.push_str(before);
    source.push_str(after);
    Ok(source)
}

#[cfg(test)]
mod tests;
