//! Complete actual original Vue2 outputs; compiler goldens are independent.

use serde::Deserialize;
use vize_glyph::native_doc::{LineEnding, PrintOptions, print, vue2_text_document};
use vize_l0::{Allocator, String};
use vize_l1::dialect::vue2::surface;

#[path = "native_vue2_text_document/custody.rs"]
mod custody;
#[path = "native_vue2_text_document/refusals.rs"]
mod refusals;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: std::string::String,
    source: std::string::String,
    width: usize,
    line_ending: std::string::String,
    expected: std::string::String,
}
#[derive(Deserialize)]
struct Packet {
    cases: std::vec::Vec<Case>,
}

fn format(source: &str, options: PrintOptions) -> String {
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, source).unwrap();
    let view = owner.text_for(owner.children().next().unwrap()).unwrap();
    let document = vue2_text_document(view, &arena).unwrap();
    print(document.document(), &options)
}

#[test]
fn whole_print_goldens_and_idempotence_capture_only_real_original_docs() {
    let packet: Packet =
        serde_json::from_str(include_str!("fixtures/native-vue2-text-doc-2.7.16.json")).unwrap();
    assert_eq!(packet.cases.len(), 48);
    let mut captures = std::vec::Vec::new();
    for case in packet.cases {
        let options = PrintOptions {
            width: case.width,
            line_ending: match case.line_ending.as_str() {
                "Lf" => LineEnding::Lf,
                "CrLf" => LineEnding::CrLf,
                _ => panic!("known complete options"),
            },
            ..PrintOptions::default()
        };
        let printed = format(&case.source, options);
        assert_eq!(printed.as_str(), case.expected, "{}", case.id);
        assert_eq!(
            format(&printed, options),
            printed,
            "idempotence {}",
            case.id
        );
        captures.push(serde_json::json!({"id":case.id,"source":case.source,"width":case.width,"lineEnding":case.line_ending,"printed":printed.as_str()}));
    }
    if let Some(path) = std::env::var_os("VIZE_GLYPH_VUE2_TEXT_CAPTURE") {
        std::fs::write(path, serde_json::to_vec(&serde_json::json!({"schema":"vize.native-vue2-text-document-capture","version":1,"sourceHead":std::env::var("VIZE_GLYPH_VUE2_TEXT_SOURCE_HEAD").ok(),"executionCommit":std::env::var("GITHUB_SHA").ok(),"workflowRun":std::env::var("GITHUB_RUN_ID").ok(),"cases":captures})).unwrap()).unwrap();
    }
}

#[test]
fn full_authored_gaps_keep_crlf_trim_unicode_names_and_list_distinctions() {
    for (source, expected) in [
        ("{{ a+b\r\n | upper }}", "{{ a + b\r\n | upper }}"),
        (
            "{{ a | upper(\t\n\u{00a0}) }}",
            "{{ a | upper(\t\n\u{00a0}) }}",
        ),
        (
            "{{ a | upper(&#160;&#xfeff;) }}",
            "{{ a | upper(&#160;&#xfeff;) }}",
        ),
        ("{{ a | with-dash }}", "{{ a | with-dash }}"),
        ("{{ a | 后缀(2+3, '後',) }}", "{{ a | 后缀(2 + 3, '後',) }}"),
        ("{{a+b|add(2+3)}}", "{{a + b|add(2 + 3)}}"),
    ] {
        assert_eq!(format(source, PrintOptions::default()), expected);
        assert_eq!(format(expected, PrintOptions::default()), expected);
    }
}
