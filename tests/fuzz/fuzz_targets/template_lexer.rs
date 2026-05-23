#![no_main]

// Vue template tokenizer/lexer fuzz target.
//
// Drives the low-level `vize_armature::Tokenizer` state machine directly with
// arbitrary UTF-8 template input. The parser fuzz target covers callback-driven
// AST construction; this target isolates tokenizer state transitions and
// recovery from malformed tags, attributes, entities, comments, and
// interpolations.
//
// The corpus is seeded from the `<template>` blocks of repository .vue
// fixtures by `tools/fuzz/seed_corpus.mjs`.
use libfuzzer_sys::fuzz_target;
use vize_armature::{Callbacks, ErrorCode, QuoteType, Tokenizer};

#[derive(Default)]
struct SinkCallbacks;

impl Callbacks for SinkCallbacks {
    fn on_text(&mut self, _start: usize, _end: usize) {}
    fn on_text_entity(&mut self, _char: char, _start: usize, _end: usize) {}
    fn on_interpolation(&mut self, _start: usize, _end: usize) {}
    fn on_open_tag_name(&mut self, _start: usize, _end: usize) {}
    fn on_open_tag_end(&mut self, _end: usize) {}
    fn on_self_closing_tag(&mut self, _end: usize) {}
    fn on_close_tag(&mut self, _start: usize, _end: usize) {}
    fn on_attrib_data(&mut self, _start: usize, _end: usize) {}
    fn on_attrib_entity(&mut self, _char: char, _start: usize, _end: usize) {}
    fn on_attrib_end(&mut self, _quote: QuoteType, _end: usize) {}
    fn on_attrib_name(&mut self, _start: usize, _end: usize) {}
    fn on_attrib_name_end(&mut self, _end: usize) {}
    fn on_dir_name(&mut self, _start: usize, _end: usize) {}
    fn on_dir_arg(&mut self, _start: usize, _end: usize) {}
    fn on_dir_modifier(&mut self, _start: usize, _end: usize) {}
    fn on_comment(&mut self, _start: usize, _end: usize) {}
    fn on_cdata(&mut self, _start: usize, _end: usize) {}
    fn on_processing_instruction(&mut self, _start: usize, _end: usize) {}
    fn on_end(&mut self) {}
    fn on_error(&mut self, _code: ErrorCode, _index: usize) {}
}

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let mut tokenizer = Tokenizer::new(source, SinkCallbacks);
    tokenizer.tokenize();
});
