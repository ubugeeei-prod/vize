use super::vlq::BASE64_CHARS;
use super::*;

/// Decode a single VLQ-encoded value from `chars`, returning the value and
/// the number of base64 digits consumed. Test-only mirror of the encoder so
/// the round trip can be asserted without an external dependency.
fn decode_vlq(chars: &[u8]) -> (i64, usize) {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    let mut consumed = 0usize;
    for &c in chars {
        let digit = BASE64_CHARS.iter().position(|&b| b == c).unwrap() as u64;
        consumed += 1;
        let has_continuation = (digit & 0b10_0000) != 0;
        result |= (digit & 0b1_1111) << shift;
        shift += 5;
        if !has_continuation {
            break;
        }
    }
    let negative = (result & 1) != 0;
    let magnitude = (result >> 1) as i64;
    (if negative { -magnitude } else { magnitude }, consumed)
}

fn roundtrip(value: i64) {
    let mut s = String::default();
    encode_vlq(&mut s, value);
    let (decoded, consumed) = decode_vlq(s.as_bytes());
    assert_eq!(decoded, value, "VLQ roundtrip failed for {value}");
    assert_eq!(consumed, s.len(), "consumed != encoded len for {value}");
}

#[test]
fn vlq_roundtrip_basic() {
    for v in [
        0, 1, -1, 15, 16, -16, 17, -17, 1000, -1000, 123_456, -123_456,
    ] {
        roundtrip(v);
    }
}

#[test]
fn vlq_known_encodings() {
    // Reference values from the Source Map v3 spec / mozilla source-map.
    let mut s = String::default();
    encode_vlq(&mut s, 0);
    assert_eq!(s.as_str(), "A");
    s = String::default();
    encode_vlq(&mut s, 1);
    assert_eq!(s.as_str(), "C");
    s = String::default();
    encode_vlq(&mut s, -1);
    assert_eq!(s.as_str(), "D");
    s = String::default();
    encode_vlq(&mut s, 16);
    assert_eq!(s.as_str(), "gB");
}

#[test]
fn utf16_len_counts_surrogates() {
    assert_eq!(utf16_len("abc"), 3);
    assert_eq!(utf16_len("é"), 1); // U+00E9, single unit
    assert_eq!(utf16_len("𝟘"), 2); // U+1D7D8, surrogate pair
}

#[test]
fn resolve_generated_side_multiline() {
    let code = "line0\nlinX1\nli";
    // 'X' at byte 9 (gen line 1, col 3); 'l' of "li" at byte 12 (gen line 2, col 0)
    let segs = [
        Segment {
            generated_offset: 9,
            source_offset: 0,
            name: None,
        },
        Segment {
            generated_offset: 12,
            source_offset: 0,
            name: None,
        },
    ];
    let resolved = resolve_positions(code, "", &segs);
    assert_eq!(
        (resolved[0].generated_line, resolved[0].generated_column),
        (1, 3)
    );
    assert_eq!(
        (resolved[1].generated_line, resolved[1].generated_column),
        (2, 0)
    );
}

#[test]
fn resolve_source_offset_to_line_column() {
    // "<div>\n  {{ msg }}\n</div>": line 1 starts at byte 6, so `msg` at byte
    // offset 11 is on line 1 (0-indexed), column 5 (`  {{ ` precedes it).
    let source = "<div>\n  {{ msg }}\n</div>";
    let starts = line_start_table(source);
    assert_eq!(resolve_in_table(source, &starts, 11), (1, 5));
    // The `<` at offset 0 is line 0, column 0.
    assert_eq!(resolve_in_table(source, &starts, 0), (0, 0));
    // The `<` of `</div>` is at offset 18, line 2, column 0.
    assert_eq!(resolve_in_table(source, &starts, 18), (2, 0));
}

#[test]
fn javascript_line_terminators_resolve_both_sides_once_with_utf16_columns() {
    for ending in ["\n", "\r\n", "\r", "\u{2028}", "\u{2029}"] {
        let source = vize_l0::cstr!("a{ending}雪🌸x{ending}z");
        let code = vize_l0::cstr!("header{ending}b{ending}🌸雪x{ending}z");
        let segments = [Segment {
            generated_offset: code.find('x').unwrap() as u32,
            source_offset: source.find('x').unwrap() as u32,
            name: Some(0),
        }];
        let resolved = resolve_positions(&code, &source, &segments);
        assert_eq!(resolved.len(), 1);
        assert_eq!(
            (resolved[0].generated_line, resolved[0].generated_column),
            (2, 3)
        );
        assert_eq!((resolved[0].source_line, resolved[0].source_column), (1, 3));
        assert_eq!(
            line_start_table(&source),
            vec![0, 1 + ending.len(), 9 + 2 * ending.len()]
        );
    }
}

#[test]
fn resolve_positions_clamps_non_char_boundary_offsets() {
    let generated = "const value = \"最大\";\n";
    let source = "<template>\n  {{ 最大値 }}\n</template>";
    let generated_inside_char = generated.find("最").unwrap() + 1;
    let source_inside_char = source.find("最").unwrap() + 1;
    let segs = [Segment {
        generated_offset: generated_inside_char as u32,
        source_offset: source_inside_char as u32,
        name: None,
    }];

    let resolved = resolve_positions(generated, source, &segs);

    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].generated_column, 15);
    assert_eq!(resolved[0].source_line, 1);
    assert_eq!(resolved[0].source_column, 5);
}

#[test]
fn finish_produces_valid_v3_doc() {
    let mut b = SourceMapBuilder::new();
    // generated `_ctx.msg` at offset 0; source `msg` at byte offset 8.
    b.add_raw(0, 8);
    let code = "_ctx.msg";
    let json = b.finish(code, "template.vue", "<div>{{ msg }}</div>");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["version"], 3);
    assert_eq!(parsed["sources"][0], "template.vue");
    assert_eq!(parsed["sourcesContent"][0], "<div>{{ msg }}</div>");
    // First segment of first line: gen col 0, src 0, src line 0, src col 8.
    let mappings = parsed["mappings"].as_str().unwrap();
    let (gen_col, c1) = decode_vlq(mappings.as_bytes());
    let (src_idx, c2) = decode_vlq(&mappings.as_bytes()[c1..]);
    let (src_line, c3) = decode_vlq(&mappings.as_bytes()[c1 + c2..]);
    let (src_col, _) = decode_vlq(&mappings.as_bytes()[c1 + c2 + c3..]);
    assert_eq!((gen_col, src_idx, src_line, src_col), (0, 0, 0, 8));
}

#[test]
fn named_segment_populates_names_and_fifth_field() {
    let mut b = SourceMapBuilder::new();
    // Generated prop key `id` at offset 0; source `id` at byte offset 5.
    b.add_named(0, 5, "id");
    let code = "id: \"app\"";
    let json = b.finish(code, "Foo.vue", r#"<div id="app">"#);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    // The symbol is recorded in `names`.
    assert_eq!(parsed["names"][0], "id");

    // The single segment carries all 5 VLQ fields, the last being the
    // name-index delta (0, i.e. the first `names` entry).
    let mappings = parsed["mappings"].as_str().unwrap();
    let bytes = mappings.as_bytes();
    let (gen_col, c1) = decode_vlq(bytes);
    let (src_idx, c2) = decode_vlq(&bytes[c1..]);
    let (src_line, c3) = decode_vlq(&bytes[c1 + c2..]);
    let (src_col, c4) = decode_vlq(&bytes[c1 + c2 + c3..]);
    let consumed = c1 + c2 + c3 + c4;
    assert!(consumed < bytes.len(), "a 5th VLQ field must be present");
    let (name_idx, c5) = decode_vlq(&bytes[consumed..]);
    assert_eq!(
        (gen_col, src_idx, src_line, src_col, name_idx),
        (0, 0, 0, 5, 0)
    );
    assert_eq!(consumed + c5, bytes.len(), "no trailing bytes after name");
}

#[test]
fn intern_name_deduplicates() {
    let mut b = SourceMapBuilder::new();
    // Same symbol on two anchors interns to one `names` entry.
    b.add_named(0, 0, "id");
    b.add_named(10, 4, "id");
    b.add_named(20, 8, "class");
    let json = b.finish("0123456789012345678901234", "Foo.vue", "");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let names = parsed["names"].as_array().unwrap();
    assert_eq!(names.len(), 2, "`id` should be deduplicated");
    assert_eq!(names[0], "id");
    assert_eq!(names[1], "class");
}
