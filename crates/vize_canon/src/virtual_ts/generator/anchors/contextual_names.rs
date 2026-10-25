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

#[test]
fn every_keyword_prop_anchor_references_its_actual_generated_declaration() {
    let code = generate("all-keyword-props");
    let anchors = anchors(&code);
    for statement in anchors
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let name = statement.strip_prefix("void ").unwrap();
        assert!(
            code.contains(cstr!("const {name} = props[").as_str()),
            "anchor has no generated prop binding: {statement}\n{code}"
        );
    }
    for name in [
        "from", "of", "as", "debugger", "extends", "false", "true", "null",
    ] {
        assert!(
            anchors.contains(cstr!("void _{name};").as_str()),
            "{name}: {anchors}"
        );
        assert!(
            !anchors.contains(cstr!("void {name};").as_str()),
            "{name}: {anchors}"
        );
    }
}

#[test]
fn authored_contextual_names_and_shadowing_keep_their_lexical_identity() {
    for case in [
        "authored-contextual-locals",
        "authored-prop-shadow-imports",
        "reactive-prop-destructure",
    ] {
        let code = generate(case);
        let anchors = anchors(&code);
        assert!(
            anchors.contains("void from;") || anchors.contains("keyof typeof from"),
            "{case}: {anchors}"
        );
        assert!(!anchors.contains("void _from;"), "{case}: {anchors}");
    }
    let code = generate("const-enum-import");
    assert!(anchors(&code).contains("void (null as unknown as keyof typeof from);"));
    assert!(!anchors(&code).contains("typeof _from"));
}

#[test]
fn unicode_prop_and_authored_local_anchors_keep_the_existing_identifier_authority() {
    let code = generate("unicode-identities");
    for name in [
        "café",
        "caf_",
        "日本語",
        "𐐀value",
        "a\u{200c}b",
        "$value",
        "_private",
        "résumé",
    ] {
        assert!(
            anchors(&code).contains(cstr!("void {name};").as_str()),
            "{name}: {}",
            anchors(&code)
        );
    }
}
