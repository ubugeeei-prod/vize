//! Davinci microbenches: tokenizer and parser over the P0-2 fixture ladder.
//!
//! Run with: cargo bench -p vize_armature --bench davinci
//!
//! Each case exports wall p50/p95, allocation calls, peak transient heap, and
//! the RSS delta to `bench/results/davinci/<bench_id>.json` via
//! `davinci_harness`. Inputs are the root `<template>` block contents of the
//! committed ladder fixtures, matching what the SFC pipeline hands armature.
//! The tokenize cases drive the tokenizer with no-op callbacks, so they price
//! raw tokenization; the parse cases include a fresh arena per iteration, the
//! same allocation pattern as one production template compile.

use criterion::{Criterion, criterion_group};
use davinci_harness::fixtures::{LADDER, template_block};
use vize_armature::{Callbacks, ErrorCode, Parser, QuoteType, Tokenizer};
use vize_s0::{Allocator, cstr};

/// No-op tokenizer sink: every callback discards its span, so the measured
/// cost is the tokenizer state machine alone.
struct NoopCallbacks;

impl Callbacks for NoopCallbacks {
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

fn davinci(criterion: &mut Criterion) {
    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");

        let tokenize_id = cstr!("armature_tokenize_{}", fixture.name);
        davinci_harness::bench_with_metrics(criterion, &tokenize_id, fixture.relative_path, || {
            let mut tokenizer = Tokenizer::new(template, NoopCallbacks);
            tokenizer.tokenize();
        });

        let parse_id = cstr!("armature_parse_{}", fixture.name);
        davinci_harness::bench_with_metrics(criterion, &parse_id, fixture.relative_path, || {
            let allocator = Allocator::new();
            let (root, errors) = Parser::new(&allocator, template).parse();
            (root.children.len(), errors.len())
        });
    }
}

criterion_group!(davinci_group, davinci);
davinci_harness::main!(davinci_group);
