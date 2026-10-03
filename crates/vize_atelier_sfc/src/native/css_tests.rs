use super::{NativeSfcCompileError, NativeSfcCompileOptions, compile_native_sfc};
use crate::{SfcCompileOptions, SfcParseOptions, StyleCompileOptions, compile_sfc, parse_sfc};
use vize_l0::{Allocator, Span};

#[test]
fn complete_plain_css_result_matches_the_existing_result_shape_and_original_bytes() {
    for (styles, raw, trimmed) in [
        ("", None, None),
        ("<style></style>", None, None),
        ("<style> \r\n\t </style>", Some(" \r\n\t "), None),
        (
            "<style lang=css>.red{color:red}</style>",
            Some(".red{color:red}"),
            Some(".red{color:red}"),
        ),
        (
            "<style></style><style> .red{color:red} </style><style></style>",
            Some(" .red{color:red} \n"),
            Some(".red{color:red}\n"),
        ),
        (
            "<style> .red{color:red} </style><style> \r\n\t </style>",
            Some(" .red{color:red} \n \r\n\t "),
            Some(".red{color:red}\n"),
        ),
        (
            "<style>/* original &amp; 雪🌸 */\n.a{color:red}</style><style lang='css'>\n.b{content:'雪🌸'}\n</style>",
            Some("/* original &amp; 雪🌸 */\n.a{color:red}\n\n.b{content:'雪🌸'}\n"),
            Some("/* original &amp; 雪🌸 */\n.a{color:red}\n.b{content:'雪🌸'}"),
        ),
    ] {
        for (trim, expected) in [(false, raw), (true, trimmed)] {
            let arena = Allocator::default();
            let source = format!("<!-- original 雪🌸 -->{styles}<template><p/></template>");
            let native = compile_native_sfc(
                &arena,
                &source,
                NativeSfcCompileOptions {
                    style_trim: trim,
                    ..NativeSfcCompileOptions::default()
                },
            );
            let output = native.result().unwrap();
            assert_eq!(output.css(), expected, "trim={trim}: {source}");
            assert_eq!(
                output.css_document().map(|document| document.as_str()),
                expected
            );
            assert!(output.css_source_map().is_none());
            assert!(
                output
                    .css_document()
                    .is_none_or(|document| document.links().is_empty())
            );
            let legacy_descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
            let legacy = compile_sfc(
                &legacy_descriptor,
                SfcCompileOptions {
                    style: StyleCompileOptions {
                        trim,
                        ..StyleCompileOptions::default()
                    },
                    ..SfcCompileOptions::default()
                },
            )
            .unwrap();
            assert_eq!(output.css(), legacy.css.as_deref());
            let base = compile_native_sfc(
                &arena,
                "<template><p/></template>",
                NativeSfcCompileOptions::default(),
            );
            assert_eq!(output.code(), base.result().unwrap().code());
            assert!(core::ptr::eq(
                native.observation().descriptor().source(),
                source.as_str()
            ));
            assert!(native.observation().admitted().is_some());
        }
    }
}

#[test]
fn original_setup_and_css_outputs_preserve_their_shared_source_owners() {
    let arena = Allocator::default();
    let base = "<script setup>let count=1</script><template><p>{{count}}</p></template>";
    let source = format!("<style>/* 雪🌸 */p{{color:red}}</style>{base}");
    let compilation = compile_native_sfc(
        &arena,
        &source,
        NativeSfcCompileOptions {
            source_map: true,
            ..NativeSfcCompileOptions::default()
        },
    );
    let reference = compile_native_sfc(&arena, base, NativeSfcCompileOptions::default());
    let output = compilation.result().unwrap();
    assert_eq!(output.code(), reference.result().unwrap().code());
    assert_eq!(output.css(), Some("/* 雪🌸 */p{color:red}"));
    let observation = compilation.observation();
    let descriptor = observation.descriptor().admitted().unwrap();
    assert_eq!(descriptor.styles().len(), 1);
    assert_eq!(descriptor.setup().unwrap().block().source(), "let count=1");
    assert!(observation.admitted().is_some());
    assert!(core::ptr::eq(
        observation.file().unwrap().file().artifact().source(),
        source.as_str()
    ));
    let script = observation.scripts().first().unwrap();
    assert!(
        output
            .document()
            .links()
            .iter()
            .any(|link| link.authored == script.block().span())
    );
    for map in [
        output.source_map().unwrap(),
        output.css_source_map().unwrap(),
    ] {
        let map: serde_json::Value = serde_json::from_str(map).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    }
}

#[test]
fn mapped_and_unmapped_css_preserve_original_whole_file_coordinates() {
    let source = "<!-- 雪🌸 -->\r\n<style lang=css>\r\n /* &amp;雪🌸 */\r\n .a{color:red} \r\n</style><template><p/></template>\r\n<style>\n .b{content:'雪🌸'} \n</style>";
    for trim in [false, true] {
        let arena = Allocator::default();
        let plain = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                style_trim: trim,
                ..NativeSfcCompileOptions::default()
            },
        );
        let mapped = compile_native_sfc(
            &arena,
            source,
            NativeSfcCompileOptions {
                filename: "Native雪🌸.vue",
                source_map: true,
                style_trim: trim,
                ..NativeSfcCompileOptions::default()
            },
        );
        let output = mapped.result().unwrap();
        assert_eq!(output.code(), plain.result().unwrap().code());
        assert_eq!(output.css(), plain.result().unwrap().css());
        let document = output.css_document().unwrap();
        let styles = mapped.observation().descriptor().admitted().unwrap();
        assert_eq!(styles.styles().len(), 2);
        let map: serde_json::Value =
            serde_json::from_str(output.css_source_map().unwrap()).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(map["sources"], serde_json::json!(["Native雪🌸.vue"]));
        assert_eq!(map["file"], "Native雪🌸.vue");
        assert_eq!(map["names"], serde_json::json!([]));
        let segments = decode_map(map["mappings"].as_str().unwrap());
        assert_eq!(segments.len(), document.links().len());
        for (link, segment) in document.links().iter().zip(segments) {
            let generated = document
                .as_str()
                .get(link.generated.start as usize..link.generated.end as usize)
                .unwrap();
            let authored = source
                .get(link.authored.start as usize..link.authored.end as usize)
                .unwrap();
            assert_eq!(generated, authored);
            assert!(styles.styles().any(|style| {
                let span = style.block().span();
                span.start <= link.authored.start && span.end >= link.authored.end
            }));
            assert_eq!(
                segment,
                (
                    position(document.as_str(), link.generated.start),
                    position(source, link.authored.start),
                )
            );
        }
    }
}

#[test]
fn conservative_binding_refusals_keep_exact_original_spans_and_every_owner() {
    for (css, evidence) in [
        ("p{color:v-bind(color)}", "v-bind"),
        ("p{content:'V-BIND(color)'}", "V-BIND"),
        ("/* v-bind(color) */p{color:red}", "v-bind"),
        ("p{color:v-/**/bind(color)}", "v-/**/bind"),
        (
            "p{color:v/**/-/**/b/**/i/**/n/**/d(color)}",
            "v/**/-/**/b/**/i/**/n/**/d",
        ),
        (r"p{content:'\76 -bind(color)'}", "\\"),
        ("p{color:red}/* unfinished", "/* unfinished"),
    ] {
        let arena = Allocator::default();
        let source = format!(
            "<!-- 雪🌸 --><style>{css}</style><template><p/></template><style>.retained{{color:red}}</style>"
        );
        let compilation = compile_native_sfc(&arena, &source, NativeSfcCompileOptions::default());
        let start = source.find(evidence).unwrap() as u32;
        assert_eq!(
            compilation.result().unwrap_err(),
            NativeSfcCompileError::StyleBindSyntaxUnproven {
                container_index: 0,
                span: Span::new(start, start + evidence.len() as u32),
            }
        );
        let descriptor = compilation.observation().descriptor().admitted().unwrap();
        assert_eq!(descriptor.styles().len(), 2);
        assert_eq!(descriptor.styles().next().unwrap().block().source(), css);
        assert!(core::ptr::eq(descriptor.source(), source.as_str()));
        assert!(compilation.observation().admitted().is_some());
        assert!(compilation.observation().template().is_some());
        assert!(compilation.observation().file().is_some());
    }
}

fn position(source: &str, offset: u32) -> (i32, i32) {
    let prefix = source.get(..offset as usize).unwrap();
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() as i32;
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() as i32;
    (line, column)
}

/// Independent Source Map v3 decoder for the actual CSS output, without using
/// the production writer's coordinate or VLQ implementation.
fn decode_map(source: &str) -> Vec<((i32, i32), (i32, i32))> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let (mut original_line, mut original_column, mut source_index) = (0, 0, 0);
    let mut output = Vec::new();
    for (line, text) in source.split(';').enumerate() {
        let mut column = 0;
        for segment in text.split(',').filter(|segment| !segment.is_empty()) {
            let (mut value, mut shift) = (0, 0);
            let mut fields = Vec::new();
            for byte in segment.bytes() {
                let digit = alphabet.iter().position(|entry| *entry == byte).unwrap() as i32;
                value |= (digit & 31) << shift;
                if digit & 32 == 0 {
                    fields.push(if value & 1 == 0 {
                        value >> 1
                    } else {
                        -(value >> 1)
                    });
                    value = 0;
                    shift = 0;
                } else {
                    shift += 5;
                }
            }
            assert_eq!(shift, 0);
            assert_eq!(fields.len(), 4);
            column += fields[0];
            source_index += fields[1];
            original_line += fields[2];
            original_column += fields[3];
            assert_eq!(source_index, 0);
            output.push(((line as i32, column), (original_line, original_column)));
        }
    }
    output
}
