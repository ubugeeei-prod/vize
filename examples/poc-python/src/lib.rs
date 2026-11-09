//! Proof of concept: a Vue component scripted in Python, compiled
//! through the level crates.
//!
//! Python sits on the Lang axis: the framework stays Vue. The L1 Vue
//! container and markup parser read the file, the L1 to L2 foreign-expression
//! hook admits every template expression as a `python` expression with
//! [`lang::Python`] as its capability, the shared L2 to L3 lowering runs, and
//! the shared Vue render target asks the same capability to emit JavaScript.
//! [`lang::script`] translates the `<script setup lang="python">` block.
//! Not a supported product surface (2026-11-03 scope decision).

mod lang;

use poc_support::Error;
use vize_l0::String;

/// Compile one Vue SFC with `<script setup lang="python">` to a JavaScript module.
pub fn compile(source: &str) -> Result<String, Error> {
    poc_support::vue::compile::<lang::Python>(source, "python", lang::script)
}

#[cfg(test)]
mod tests {
    use super::compile;

    const COUNTER: &str = include_str!("../input/Counter.vue");
    const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/__snapshots__/Counter.js");

    #[test]
    fn counter_compiles_to_a_vue_component_module() {
        let output = compile(COUNTER);
        assert!(output.is_ok(), "{output:?}");
        let output = output.unwrap_or_default();
        assert_eq!(poc_support::check_module(&output), Ok(()));
        assert_eq!(poc_support::snapshot(SNAPSHOT, &output), Ok(()));
    }

    #[test]
    fn uncovered_python_is_refused_with_its_span() {
        let source =
            "<script setup lang=\"python\">\nwhile x:\n</script>\n<template><p /></template>\n";
        let refused = compile(source);
        assert_eq!(refused.map_err(|error| error.span.start), Err(29));
    }
}
