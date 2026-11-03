//! Full generation owns the value-read list; only exact imported names reuse it.

use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
use crate::virtual_ts::VirtualTsOptions;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde::Deserialize;
use std::path::Path;
use vize_carton::String;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    source: String,
    expected_value_reads: Vec<String>,
    expected_binding_anchors: Vec<String>,
}

#[derive(Deserialize)]
struct Controls {
    schema: String,
    cases: Vec<Case>,
}

fn block_lines<'a>(code: &'a str, header: &str) -> Vec<&'a str> {
    code.lines()
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.is_empty() && !line.starts_with("  })();"))
        .collect()
}

#[test]
fn exact_component_value_reads_reuse_only_their_own_import_anchors() {
    let controls: Controls = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/component-value-anchor-reuse/input.json"
    )))
    .unwrap();
    assert_eq!(controls.schema, "vize.component-value-anchor-reuse.v1");
    assert_eq!(controls.cases.len(), 7);
    for case in controls.cases {
        let output = generate_vue_document_virtual_ts(
            Path::new("Anchor.vue"),
            &case.source,
            &VirtualTsOptions::default(),
            &ImportRewriter::new(),
            false,
        )
        .unwrap();
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &output.pre_rewrite_code, SourceType::ts()).parse();
        assert_eq!(parsed.diagnostics.len(), 0, "{}", case.id);
        assert_eq!(
            block_lines(
                &output.pre_rewrite_code,
                "  // Mark used components as referenced"
            ),
            case.expected_value_reads
                .iter()
                .map(|line| line.as_str())
                .collect::<Vec<_>>(),
            "{}: whole value-read block",
            case.id
        );
        assert_eq!(
            block_lines(
                &output.pre_rewrite_code,
                "  // Reference setup bindings (used in template/CSS v-bind)"
            ),
            case.expected_binding_anchors
                .iter()
                .map(|line| line.as_str())
                .collect::<Vec<_>>(),
            "{}: whole binding-anchor block",
            case.id
        );
    }
}
