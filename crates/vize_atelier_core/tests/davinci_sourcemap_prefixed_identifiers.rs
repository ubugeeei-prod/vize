//! Davinci P1-9: exact source-map assertions for rewritten identifiers
//! over the P0 fixture ladder.
//!
//! The identifier-prefix transform is span-preserving: the AST decides
//! which identifier spans are rewritten and the output is spliced into the
//! original expression text, so a node's `loc` still names the exact
//! template bytes its generated expression came from. These tests pin that
//! property end to end: for a representative rewritten identifier per
//! ladder fixture, the source-map segment anchored at the generated
//! `_ctx.<name>` position must map back to the identifier's template
//! position — asserted with exact equality on the mapping tuples,
//! following the source-map test pattern in `codegen/tests.rs`.
//!
//! Coverage note: event-handler *values* are emitted by
//! `generate_event_handler`, which records no mapping segment today (the
//! #1533 fidelity scope) — `medium`'s only rewritten identifiers are
//! handler values, so its test pins that gap exactly instead of dodging
//! the fixture silently.

use davinci_harness::fixtures::{LADDER, template_block};
use vize_atelier_core::{CodegenOptions, CodegenResult, ErrorCode, TransformOptions, generate};

/// A single decoded `mappings` segment: 0-indexed generated line/column and
/// the source line/column it points back to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DecodedSegment {
    generated_line: u32,
    generated_column: u32,
    source_line: u32,
    source_column: u32,
}

const VLQ_CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Decode one base64-VLQ value, returning the value and the number of
/// base64 digits consumed. Independent decoder so the test does not lean on
/// the encoder it validates.
fn decode_one_vlq(bytes: &[u8]) -> (i64, usize) {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    let mut consumed = 0usize;
    for &c in bytes {
        let digit = VLQ_CHARS
            .iter()
            .position(|&b| b == c)
            .expect("valid base64") as u64;
        consumed += 1;
        result |= (digit & 0b1_1111) << shift;
        shift += 5;
        if digit & 0b10_0000 == 0 {
            break;
        }
    }
    let negative = result & 1 != 0;
    let magnitude = (result >> 1) as i64;
    (if negative { -magnitude } else { magnitude }, consumed)
}

/// Decode a full v3 `mappings` string into absolute decoded segments (the
/// optional 5th name field is consumed but not kept — these anchors are
/// anonymous expression mappings).
fn decode_mappings(mappings: &str) -> Vec<DecodedSegment> {
    let mut out = Vec::new();
    let mut source_line = 0i64;
    let mut source_column = 0i64;

    for (generated_line, line) in mappings.split(';').enumerate() {
        let mut generated_column = 0i64;
        for seg in line.split(',').filter(|s| !s.is_empty()) {
            let bytes = seg.as_bytes();
            let (d_gen_col, c1) = decode_one_vlq(bytes);
            let (_d_src_idx, c2) = decode_one_vlq(&bytes[c1..]);
            let (d_src_line, c3) = decode_one_vlq(&bytes[c1 + c2..]);
            let (d_src_col, _c4) = decode_one_vlq(&bytes[c1 + c2 + c3..]);
            generated_column += d_gen_col;
            source_line += d_src_line;
            source_column += d_src_col;
            out.push(DecodedSegment {
                generated_line: generated_line as u32,
                generated_column: generated_column as u32,
                source_line: source_line as u32,
                source_column: source_column as u32,
            });
        }
    }
    out
}

/// 0-indexed (line, column) of byte offset `byte_idx` in `text`, columns in
/// UTF-16 code units per the source-map convention.
fn position_of(text: &str, byte_idx: usize) -> (u32, u32) {
    let prefix = &text[..byte_idx];
    let line = prefix.bytes().filter(|&b| b == b'\n').count() as u32;
    let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let column = text[line_start..byte_idx]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum();
    (line, column)
}

/// Byte offsets of every occurrence of `needle` in `haystack`.
fn occurrences(haystack: &str, needle: &str) -> Vec<usize> {
    haystack.match_indices(needle).map(|(i, _)| i).collect()
}

/// Byte offset of the single occurrence of `needle` in `haystack`; the
/// occurrence count is asserted exactly so every anchor is unambiguous.
fn unique_offset(haystack: &str, needle: &str, what: &str) -> usize {
    let found = occurrences(haystack, needle);
    assert_eq!(
        found.len(),
        1,
        "{what} needle {needle:?} must occur exactly once"
    );
    found[0]
}

fn ladder_template(name: &str) -> &'static str {
    let fixture = LADDER
        .iter()
        .find(|fixture| fixture.name == name)
        .expect("ladder fixture name");
    template_block(fixture.source).expect("every ladder fixture has a template block")
}

/// Compile a ladder template with `prefix_identifiers` and a source map,
/// mirroring the `compile_with_map` pattern in `codegen/tests.rs`. The
/// real-project fixtures carry `<foo />` self-closing compat rewrites;
/// those surface as recoverable `ExtendPoint` notes, so only real errors
/// are asserted away.
fn compile_with_map(src: &str, filename: &str) -> CodegenResult {
    let allocator = vize_s0::Allocator::new();
    let (mut root, errors) = vize_atelier_core::parse(&allocator, src);
    let fatal: Vec<_> = errors
        .iter()
        .filter(|error| error.code != ErrorCode::ExtendPoint)
        .collect();
    assert_eq!(fatal.len(), 0, "Parse errors: {fatal:?}");
    vize_atelier_core::transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    generate(
        &root,
        CodegenOptions {
            prefix_identifiers: true,
            source_map: true,
            filename: filename.into(),
            ..Default::default()
        },
    )
}

/// One rewritten-identifier anchor: `template_needle` occurs exactly once
/// in the fixture template and the identifier under test starts
/// `template_offset` bytes into it; `generated_needle` occurs exactly once
/// in the generated code and the rewritten identifier's recorded anchor
/// byte (its `_ctx.` first byte) sits `generated_offset` bytes into it.
struct Anchor {
    template_needle: &'static str,
    template_offset: usize,
    generated_needle: &'static str,
    generated_offset: usize,
}

fn assert_anchor_maps_exactly(fixture_name: &str, anchor: &Anchor) {
    let template = ladder_template(fixture_name);
    let result = compile_with_map(template, "Fixture.vue");
    let map = result
        .map
        .expect("map should be Some when source_map is on");
    let parsed: serde_json::Value = serde_json::from_str(&map).expect("map must be valid JSON");
    let segments = decode_mappings(parsed["mappings"].as_str().expect("mappings string"));

    let source_offset =
        unique_offset(template, anchor.template_needle, "template") + anchor.template_offset;
    let expected_source = position_of(template, source_offset);

    let generated_offset =
        unique_offset(&result.code, anchor.generated_needle, "generated") + anchor.generated_offset;
    let (gen_line, gen_col) = position_of(&result.code, generated_offset);

    let anchored: Vec<&DecodedSegment> = segments
        .iter()
        .filter(|s| s.generated_line == gen_line && s.generated_column == gen_col)
        .collect();
    assert_eq!(
        anchored.len(),
        1,
        "{fixture_name}: exactly one segment must anchor {:?} at generated {}:{}",
        anchor.generated_needle,
        gen_line,
        gen_col
    );
    assert_eq!(
        (anchored[0].source_line, anchored[0].source_column),
        expected_source,
        "{fixture_name}: generated {:?} must map back to the template occurrence of its identifier",
        anchor.generated_needle
    );
}

#[test]
fn small_label_interpolation_maps_exactly() {
    // `{{ label }}` -> `_toDisplayString(_ctx.label)`.
    assert_anchor_maps_exactly(
        "small",
        &Anchor {
            template_needle: "{{ label }}",
            template_offset: 3,
            generated_needle: "_ctx.label",
            generated_offset: 0,
        },
    );
}

#[test]
fn small_doubled_interpolation_maps_exactly() {
    assert_anchor_maps_exactly(
        "small",
        &Anchor {
            template_needle: "{{ doubled }}",
            template_offset: 3,
            generated_needle: "_ctx.doubled",
            generated_offset: 0,
        },
    );
}

#[test]
fn large_bound_prop_value_maps_exactly() {
    // `:chartElement="dashboardContainer"`: the prop-key needle keeps the
    // generated occurrence unique (the identifier also appears inside a
    // larger `:style` expression).
    assert_anchor_maps_exactly(
        "large",
        &Anchor {
            template_needle: ":chartElement=\"dashboardContainer\"",
            template_offset: 15,
            generated_needle: "chartElement: _ctx.dashboardContainer",
            generated_offset: 14,
        },
    );
}

#[test]
fn stress_deep_condition_maps_exactly() {
    // `v-if="level8 > 0"`: the trailing ` > 0` keeps the needle from
    // matching the deeper `level80` reference on either side.
    assert_anchor_maps_exactly(
        "stress-deep",
        &Anchor {
            template_needle: "v-if=\"level8 > 0\"",
            template_offset: 6,
            generated_needle: "_ctx.level8 > 0",
            generated_offset: 0,
        },
    );
}

#[test]
fn stress_deep_bound_class_maps_exactly() {
    // `:class="cls4"` -> `_normalizeClass(_ctx.cls4)`; the closing paren
    // keeps the needle from matching `_ctx.cls44`.
    assert_anchor_maps_exactly(
        "stress-deep",
        &Anchor {
            template_needle: ":class=\"cls4\"",
            template_offset: 8,
            generated_needle: "_ctx.cls4)",
            generated_offset: 0,
        },
    );
}

#[test]
fn stress_wide_bound_attribute_maps_exactly() {
    assert_anchor_maps_exactly(
        "stress-wide",
        &Anchor {
            template_needle: ":bound-25=\"expr25\"",
            template_offset: 11,
            generated_needle: "_ctx.expr25",
            generated_offset: 0,
        },
    );
}

#[test]
fn stress_interp_interpolation_maps_exactly() {
    assert_anchor_maps_exactly(
        "stress-interp",
        &Anchor {
            template_needle: "{{ item123 }}",
            template_offset: 3,
            generated_needle: "_ctx.item123",
            generated_offset: 0,
        },
    );
}

#[test]
fn medium_handler_values_are_rewritten_but_unanchored() {
    // `medium`'s only rewritten identifiers are its four
    // `handleSetLineChartData(...)` handler values, and handler values are
    // emitted without a mapping segment today (#1533 fidelity scope). Pin
    // both facts exactly: widening handler mapping fidelity must turn this
    // into a real anchor test above.
    let template = ladder_template("medium");
    let result = compile_with_map(template, "Fixture.vue");
    let map = result
        .map
        .expect("map should be Some when source_map is on");
    let parsed: serde_json::Value = serde_json::from_str(&map).expect("map must be valid JSON");
    let segments = decode_mappings(parsed["mappings"].as_str().expect("mappings string"));

    let handlers = occurrences(&result.code, "_ctx.handleSetLineChartData");
    assert_eq!(
        handlers.len(),
        4,
        "medium must rewrite its four handler values"
    );
    let all_prefixed = occurrences(&result.code, "_ctx.");
    assert_eq!(
        all_prefixed, handlers,
        "handler values must be medium's only rewritten identifiers"
    );
    for offset in handlers {
        let (gen_line, gen_col) = position_of(&result.code, offset);
        let anchored = segments
            .iter()
            .filter(|s| s.generated_line == gen_line && s.generated_column == gen_col)
            .count();
        assert_eq!(
            anchored, 0,
            "handler value at generated {gen_line}:{gen_col} is expected to carry no segment"
        );
    }
}
