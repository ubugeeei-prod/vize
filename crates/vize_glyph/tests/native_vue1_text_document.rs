//! Whole independent LF goldens, original preparation and genuine idempotence.

use serde::Deserialize;
use serde_json::{Value, json};
use vize_glyph::native_doc::{LineEnding, PrintOptions, vue1_text_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::dialect::vue1::surface;
use vize_l1::embed::EmbedSource;

#[path = "native_vue1_text_document/custody.rs"]
mod custody;
#[path = "native_vue1_text_document/refusals.rs"]
mod refusals;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: std::string::String,
    source: std::string::String,
    block_span: [u32; 2],
    path: std::vec::Vec<usize>,
    callback_span: [u32; 2],
    callback: std::string::String,
    width: usize,
    indent_width: usize,
    line_ending: std::string::String,
    expected: std::string::String,
}

#[derive(Deserialize)]
struct Packet {
    cases: std::vec::Vec<Case>,
}

fn child_at<'o, 'a>(
    owner: &'o surface::ComponentParse<'a>,
    path: &[usize],
) -> surface::TextChild<'o, 'a> {
    let mut child = owner.children().nth(path[0]).unwrap();
    for ordinal in &path[1..] {
        child = child.children().unwrap().nth(*ordinal).unwrap();
    }
    child
}

fn range(span: Span) -> [u32; 2] {
    [span.start, span.end]
}

fn map(source: EmbedSource<'_>) -> Option<std::vec::Vec<Value>> {
    source.decode_map().map(|map| {
        map.segments()
            .iter()
            .map(|segment| {
                json!({
                    "decoded": range(segment.decoded()),
                    "authored": range(segment.authored()),
                    "kind": std::format!("{:?}", segment.kind()),
                })
            })
            .collect()
    })
}

fn capture(case: &Case) -> Value {
    let arena = Allocator::default();
    let block = SourceRoot::new(&case.source)
        .unwrap()
        .block(
            &case.source[case.block_span[0] as usize..case.block_span[1] as usize],
            case.block_span[0],
        )
        .unwrap();
    let owner = surface::parse_component_block(&arena, block);
    let child = child_at(&owner, &case.path);
    let view = match owner.text_for(child) {
        Ok(view) => view,
        Err(error) => {
            return json!({"id":case.id,"source":case.source,"error":std::format!("{error:?}")});
        }
    };
    let binding = view.binding();
    let callback_span = binding.span();
    let callback = callback_span.slice(&case.source);
    let prepared = binding.syntax().unwrap().source();
    let document = match vue1_text_document(view, &arena) {
        Ok(document) => document,
        Err(error) => {
            return json!({"id":case.id,"source":case.source,"error":std::format!("{error:?}")});
        }
    };
    let options = PrintOptions {
        width: case.width,
        indent_width: case.indent_width,
        line_ending: LineEnding::Lf,
    };
    let printed = document.print(&options).unwrap();
    // This is an independent new development-only original construction over
    // the actual returned String, not a reparse in the production consumer.
    let formatted_owner = surface::parse_component(&arena, printed.as_str()).unwrap();
    let formatted_view = match formatted_owner.text_for(formatted_owner.children().next().unwrap())
    {
        Ok(view) => view,
        Err(error) => {
            return json!({"id":case.id,"source":case.source,"printed":printed.as_str(),"error":std::format!("{error:?}")});
        }
    };
    let formatted = formatted_view.binding().syntax().unwrap().source();
    let formatted_document = match vue1_text_document(formatted_view, &arena) {
        Ok(document) => document,
        Err(error) => {
            return json!({"id":case.id,"source":case.source,"printed":printed.as_str(),"error":std::format!("{error:?}")});
        }
    };
    let idempotent = formatted_document.print(&options).unwrap();
    json!({
        "id": case.id,
        "source": case.source,
        "blockSpan": range(block.span()),
        "path": case.path,
        "callbackSpan": range(callback_span),
        "callback": callback,
        "preparedText": prepared.text(),
        "preparedSpan": range(prepared.span()),
        "preparedMap": map(prepared),
        "width": case.width,
        "indentWidth": case.indent_width,
        "lineEnding": "Lf",
        "printed": printed.as_str(),
        "idempotent": idempotent.as_str(),
        "formattedPreparedText": formatted.text(),
        "formattedPreparedSpan": range(formatted.span()),
        "formattedPreparedMap": map(formatted),
    })
}

#[test]
fn whole_lf_goldens_idempotence_and_capture_use_only_original_callback_documents() {
    let fixture = include_str!("fixtures/native-vue1-text-doc-1.0.28.json");
    let packet: Packet = serde_json::from_str(fixture).unwrap();
    let captures = packet
        .cases
        .iter()
        .map(capture)
        .collect::<std::vec::Vec<_>>();
    if let Some(path) = std::env::var_os("VIZE_GLYPH_VUE1_TEXT_CAPTURE") {
        // Preserve all real positive observations before any golden or result
        // assertion; a failed source must still leave its actual packet.
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&json!({
                "schema": "vize.native-vue1-text-document-capture",
                "version": 1,
                "sourceHead": std::env::var("VIZE_GLYPH_VUE1_TEXT_SOURCE_HEAD").ok(),
                "executionCommit": std::env::var("GITHUB_SHA").ok(),
                "workflowRun": std::env::var("GITHUB_RUN_ID").ok(),
                "executablePath": std::env::current_exe().unwrap(),
                "processId": std::process::id(),
                "cases": captures,
            }))
            .unwrap(),
        )
        .unwrap();
    }
    assert_eq!(packet.cases.len(), 64);
    for (case, actual) in packet.cases.iter().zip(&captures) {
        assert!(actual.get("error").is_none(), "{}: {actual}", case.id);
        assert_eq!(actual["blockSpan"], json!(case.block_span), "{}", case.id);
        assert_eq!(
            actual["callbackSpan"],
            json!(case.callback_span),
            "{}",
            case.id
        );
        assert_eq!(actual["callback"], case.callback, "{}", case.id);
        assert_eq!(case.line_ending, "Lf");
        assert_eq!(actual["printed"], case.expected, "{}", case.id);
        assert_eq!(actual["idempotent"], case.expected, "{}", case.id);
        assert!(!actual["printed"].as_str().unwrap().contains('\r'));
    }
}
