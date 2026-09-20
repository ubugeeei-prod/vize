use super::{
    contains_dynamic_css_module_exports, css_import_targets, css_module_block_shape,
    css_module_block_type, css_module_type, css_module_type_annotation,
    extract_authored_css_classes,
};

#[test]
fn import_targets_follow_source_order_in_both_forms() {
    assert_eq!(
        css_import_targets(
            "@import \"./a.css\";\n@import url('./b.css');\n@import url(./c.css);\n.x { }"
        ),
        vec!["./a.css", "./b.css"]
    );
    assert!(css_import_targets(".a { }").is_empty());
}

#[test]
fn extracts_only_classes_from_selector_preludes() {
    let classes = extract_authored_css_classes(
        r#"
/* .commented-out {} */
.root, .row:hover {
  background: url(./asset.png);
  content: ".not-a-class";
}
@media (width > 20rem) {
  .nested-item { color: green; }
}
"#,
    )
    .expect("plain CSS selectors should resolve");
    assert_eq!(
        classes.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
        ["nested-item", "root", "row"]
    );
    assert_eq!(
        css_module_type_annotation(&classes).as_str(),
        r#"{ "nested-item": string; "root": string; "row": string; }"#
    );
}

#[test]
fn rejects_selectors_that_cannot_form_a_closed_export_shape() {
    for source in [
        r#".root { composes : shared from "./base.css"; }"#,
        r#":global(.external) .root {}"#,
        r#".#{dynamic} {}"#,
    ] {
        assert!(contains_dynamic_css_module_exports(source), "{source}");
    }
    assert!(extract_authored_css_classes(r#".café {}"#).is_none());
    assert!(extract_authored_css_classes(r#".escaped\:name {}"#).is_none());
}

// `@import` names another stylesheet's classes: unknown without
// `resolveStyleImports`, the imported default export's type with it.
#[test]
fn imported_stylesheets_close_the_shape_only_when_resolved() {
    let source = "<style module>\n@import \"./base.css\";\n.root {}\n</style>\n";
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).expect("SFC parses");
    let style = &descriptor.styles[0];

    assert!(css_module_block_shape(style, false).is_none());
    let shape = css_module_block_shape(style, true).expect("imports are resolved");
    assert_eq!(
        css_module_type(&shape, false),
        "Record<string, string> & __VizePrettify<{} & typeof import(\"./base.css\").default & { \"root\": string; }>"
    );
    assert_eq!(
        css_module_type(&shape, true),
        "__VizePrettify<{} & typeof import(\"./base.css\").default & { \"root\": string; }>"
    );
    assert_eq!(
        css_module_block_type(style, false),
        "Record<string, string>",
        "the duplicate-name literal never resolves imports"
    );
}
