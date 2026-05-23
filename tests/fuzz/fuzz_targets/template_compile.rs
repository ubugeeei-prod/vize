#![no_main]

// Vue template → render IR fuzz target.
//
// Drives `vize_atelier_dom::compile_template` (parse + transform + codegen)
// with arbitrary UTF-8 input. Compile errors are returned in the
// `Vec<CompilerError>` channel; a panic here means the template pipeline lost
// invariants on adversarial input.
//
// The corpus is seeded from the `<template>` blocks of repository .vue
// fixtures by `tools/fuzz/seed_corpus.mjs`.
use libfuzzer_sys::fuzz_target;
use vize_atelier_dom::compile_template;
use vize_carton::Bump;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let allocator = Bump::new();
    let _ = compile_template(&allocator, source);
});
