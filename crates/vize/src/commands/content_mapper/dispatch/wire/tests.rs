use serde_json::json;
use vize_canon::{
    ContentMapperDiagnostic, ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives,
    ContentMapperSemanticLink, ContentMapperSpan, ContentMapperTransform,
    ContentMapperUnusedExpectDiagnostic,
};

use super::{encode_error, encode_result, encode_transform};

fn transform(full: bool) -> ContentMapperTransform {
    ContentMapperTransform {
        text: "const a = \"雪\\n\";\u{2028}🌸\t\r\u{1}\u{8}\u{c}\u{1f}\u{7f}\n".into(),
        extension: ".tsx",
        mappings: vec![
            ContentMapperSpan([0, 5, 10, 5, 0, 7]),
            ContentMapperSpan([5, 1, 15, 1, 1, super::SPAN_FEATURES_ALL]),
            ContentMapperSpan([6, 2, 16, 2, 0, 0]),
        ],
        semantic_links: if full {
            vec![ContentMapperSemanticLink {
                source_start: 1,
                source_length: 2,
                target_start: 3,
                target_length: 4,
                kind: "vueSetupTemplateRefUnwrap",
            }]
        } else {
            Vec::new()
        },
        diagnostic_directives: full.then(|| ContentMapperDiagnosticDirectives {
            unused_expect_directive_diagnostics: vec![ContentMapperUnusedExpectDiagnostic {
                code: 2578,
                message_text: "Unused '@ts-expect-error' directive.".into(),
            }],
            directives: vec![
                ContentMapperDiagnosticDirective([0, 0, 0, 1, 0, 0]),
                ContentMapperDiagnosticDirective([1, 2, 3, 4, 1, 0]),
            ],
        }),
        diagnostics: vec![ContentMapperDiagnostic {
            message_text: "bad \"value\"".into(),
            start: 9,
            length: 2,
            code: 100_002,
        }],
    }
}

#[test]
fn transform_frames_match_the_sorted_value_envelope() {
    for full in [true, false] {
        for id in [json!(7), json!("req-1"), json!(null)] {
            assert_eq!(
                encode_transform(&id, &transform(full)).unwrap(),
                encode_result(&id, super::wire_value(&transform(full))).unwrap(),
            );
        }
    }
}

#[test]
fn every_feature_spans_and_ignore_directives_drop_their_sixth_element() {
    let frame = encode_transform(&json!(1), &transform(true)).unwrap();
    let text = std::str::from_utf8(&frame).unwrap();
    let (_, body) = text.split_once("\r\n\r\n").unwrap();
    let value: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(
        value["result"]["mappings"],
        json!([[0, 5, 10, 5, 0, 7], [5, 1, 15, 1, 1], [6, 2, 16, 2, 0, 0]])
    );
    assert_eq!(
        value["result"]["diagnosticDirectives"]["directives"],
        json!([[0, 0, 0, 1, 0], [1, 2, 3, 4, 1, 0]])
    );
}

#[test]
fn ignores_over_whitespace_only_stay_off_the_wire() {
    let mut transform = transform(true);
    transform.text = "a  \n\t b;\r\n c".into();
    transform.diagnostic_directives = Some(ContentMapperDiagnosticDirectives {
        unused_expect_directive_diagnostics: vec![ContentMapperUnusedExpectDiagnostic {
            code: 2578,
            message_text: "Unused".into(),
        }],
        directives: vec![
            // `  \n\t `: nothing can start there.
            ContentMapperDiagnosticDirective([0, 0, 1, 6, 0, 0]),
            // ` b`: a diagnostic can start at `b`.
            ContentMapperDiagnosticDirective([0, 0, 5, 7, 0, 0]),
            // An expect still reports when unused.
            ContentMapperDiagnosticDirective([1, 2, 8, 11, 1, 0]),
            // Empty.
            ContentMapperDiagnosticDirective([0, 0, 12, 12, 0, 0]),
        ],
    });
    let frame = encode_transform(&json!(1), &transform).unwrap();
    let text = std::str::from_utf8(&frame).unwrap();
    let (_, body) = text.split_once("\r\n\r\n").unwrap();
    let value: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(
        value["result"]["diagnosticDirectives"]["directives"],
        json!([[0, 0, 5, 7, 0], [1, 2, 8, 11, 1, 0]])
    );
    assert_eq!(
        frame,
        encode_result(&json!(1), super::wire_value(&transform)).unwrap()
    );
}

#[test]
fn semantic_links_stay_off_the_wire() {
    let frame = encode_transform(&json!(1), &transform(true)).unwrap();
    let text = std::str::from_utf8(&frame).unwrap();
    let (_, body) = text.split_once("\r\n\r\n").unwrap();
    let value: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(
        value["result"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        [
            "diagnosticDirectives",
            "diagnostics",
            "extension",
            "mappings",
            "text"
        ]
    );
}

#[test]
fn json_strings_match_serde_json() {
    let alphabet: Vec<char> = (0u8..0x20)
        .map(char::from)
        .chain([
            '"', '\\', '/', '\u{7f}', 'a', ' ', 'é', '雪', '🌸', '\u{2028}',
        ])
        .collect();
    let mut state = 0x9e37_79b9_7f4a_7c15_u64;
    for length in 0..48 {
        for _ in 0..64 {
            let text: vize_l0::String = (0..length)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    let plain = !state.is_multiple_of(4);
                    if plain {
                        'x'
                    } else {
                        alphabet[(state >> 8) as usize % alphabet.len()]
                    }
                })
                .collect();
            let mut out = Vec::new();
            super::push_json_str(&mut out, &text);
            assert_eq!(
                std::str::from_utf8(&out).unwrap(),
                serde_json::to_string(&text).unwrap()
            );
        }
    }
}

#[test]
fn spans_match_serde_json() {
    let spans = vec![
        ContentMapperSpan([0, 9, 10, 99, 100, usize::MAX]),
        ContentMapperSpan([1, 12_345, 7, 0, 65_536, 4_294_967_296]),
    ];
    let mut out = Vec::new();
    super::push_spans(&mut out, &spans);
    assert_eq!(
        std::str::from_utf8(&out).unwrap(),
        serde_json::to_string(&spans).unwrap()
    );
    let mut empty = Vec::new();
    super::push_spans(&mut empty, &[]);
    assert_eq!(empty, b"[]");
}

#[test]
fn frames_carry_the_exact_body_length() {
    let frame = encode_error(&json!(1), -32603, "雪").unwrap();
    let text = std::str::from_utf8(&frame).unwrap();
    let (header, body) = text.split_once("\r\n\r\n").unwrap();
    assert_eq!(header, vize_l0::cstr!("Content-Length: {}", body.len()));
    assert_eq!(
        body,
        r#"{"error":{"code":-32603,"message":"雪"},"id":1,"jsonrpc":"2.0"}"#
    );
}
