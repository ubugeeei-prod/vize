use crate::batch::{ImportRewriter, generate_vue_document_virtual_ts};
use crate::virtual_ts::VirtualTsOptions;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::path::Path;
use vize_carton::{String, cstr};

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/contextual-prop-anchors-10661/"
);

fn generate(name: &str) -> String {
    let source =
        std::fs::read_to_string(cstr!("{CORPUS}{name}/src/Route.vue.txt").as_str()).unwrap();
    let result = generate_vue_document_virtual_ts(
        Path::new("Route.vue"),
        &source,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        false,
    )
    .unwrap();
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &result.pre_rewrite_code, SourceType::ts()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "{name}: {:?}",
        parsed.diagnostics
    );
    result.pre_rewrite_code
}

fn anchors(code: &str) -> &str {
    code.lines()
        .skip_while(|line| !line.contains("// Reference setup bindings"))
        .nth(1)
        .expect("the actual setup anchor list must exist")
}

fn exact_anchor(case: &str) {
    let code = generate(case);
    let expected =
        std::fs::read_to_string(cstr!("{CORPUS}{case}/expected-anchor.txt").as_str()).unwrap();
    assert_eq!(
        anchors(&code),
        expected.trim_end_matches('\n'),
        "complete anchor list: {case}"
    );
}

#[test]
fn every_keyword_prop_anchor_references_its_actual_generated_declaration() {
    for case in [
        "original-from",
        "all-keyword-props",
        "non-colliding-keyword-props",
        "authored-undefined-shadow",
        "authored-ts2552-retained",
    ] {
        exact_anchor(case);
    }
}

#[test]
fn authored_contextual_names_and_shadowing_keep_their_lexical_identity() {
    for case in [
        "authored-contextual-locals",
        "authored-prop-shadow-imports",
        "reactive-prop-destructure",
        "const-enum-import",
    ] {
        exact_anchor(case);
    }
}

#[test]
fn unicode_prop_and_authored_local_anchors_keep_the_existing_identifier_authority() {
    exact_anchor("unicode-identities");
}
