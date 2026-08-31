#![no_main]

// Vue template → render IR fuzz target.
//
// Drives `vize_atelier_dom::compile_template` (parse + VDOM codegen)
// with arbitrary UTF-8 input. Compile errors are returned in the
// `Vec<CompilerError>` channel; a panic here means the template pipeline lost
// invariants on adversarial input.
//
// The corpus is seeded from the `<template>` blocks of repository .vue
// fixtures by `tools/commands/ci/fuzz/seed_corpus.rs`.
use libfuzzer_sys::fuzz_target;
use vize_atelier_dom::compile_template;
use vize_s0::Allocator;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let allocator = Allocator::new();
    let _ = compile_template(&allocator, source);
});
