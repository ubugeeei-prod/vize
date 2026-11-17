//! Rows and cells written directly in a `<table>` stay as authored, as Vue
//! 3.5's compiler emits them (`table > tr`, `table > td`). Vapor keeps the
//! `tbody` the HTML parser inserts, because `innerHTML` reads its templates.

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/table-authored-rows/Layout.vue.txt"
);

fn compile(ssr: bool, vapor: bool) -> String {
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(SOURCE, parse.clone()).expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            template: TemplateCompileOptions {
                ssr,
                vapor,
                ..Default::default()
            },
            vapor,
            ..Default::default()
        },
    )
    .expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.as_str().to_owned()
}

#[test]
fn direct_table_rows_and_cells_stay_authored_like_vue() {
    assert_eq!(
        compile(true, false),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/table-authored-rows/module.expected.txt"
        )
    );

    let dom = compile(false, false);
    // Only the authored `tbody` is created; every row keeps its authored parent.
    assert_eq!(dom.matches("\"tbody\"").count(), 1, "{dom}");
    assert_eq!(dom.matches("\"tr\"").count(), 5, "{dom}");
    assert!(
        dom.contains("{ class: \"grid\" }, [\n      _createElementVNode(\"tr\""),
        "{dom}"
    );
}

#[test]
fn vapor_templates_keep_the_tbody_that_inner_html_inserts() {
    let vapor = compile(false, true);
    assert!(
        vapor.contains("<table class=\\\"grid\\\"><tbody><tr><th>Name</th></tr>"),
        "{vapor}"
    );
}
