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

// Global selectors do not export names, but cannot erase adjacent local keys
// (#9378). A known key must stay `string` with noUncheckedIndexedAccess.
#[test]
fn global_and_local_scopes_preserve_only_authored_module_exports() {
    for (css, expected) in [
        (".active:global(.loaded) {}", vec!["active"]),
        (":global(.external) .root {}", vec!["root"]),
        (":global .external .another, .root {}", vec!["root"]),
        (":global .external :local .root {}", vec!["root"]),
        (":global(.external) :local(.root) {}", vec!["root"]),
        (
            ":local(.root, .row):global(.loaded) {}",
            vec!["root", "row"],
        ),
        (".root:is(:global(.external), .row) {}", vec!["root", "row"]),
        // PostCSS CSS Modules keeps bare global mode across the function's
        // selector list, unlike the narrow functional :global(...) above.
        (".bar:not(:global .foo, .baz) {}", vec!["bar"]),
        (".root:is(:global .external, .row) {}", vec!["root"]),
        (
            ".root:is(:global .external, .row) .after {}",
            vec!["after", "root"],
        ),
        (
            ".root:is(:global .external, .row), .after {}",
            vec!["after", "root"],
        ),
        (
            ".root:is(:global .external, :local .row) .after {}",
            vec!["after", "root", "row"],
        ),
        ("[data-kind=.attribute] .root {}", vec!["root"]),
        (":global(.external) {}", vec![]),
    ] {
        let classes = extract_authored_css_classes(css).expect("static selector");
        assert_eq!(
            classes.iter().map(|name| name.as_str()).collect::<Vec<_>>(),
            expected,
            "{css}"
        );
    }
    for css in [":global(.external {}", ".root) {}", "[data-kind=.x {}"] {
        assert!(extract_authored_css_classes(css).is_none(), "{css}");
    }
}

#[test]
fn local_classes_in_global_selectors_keep_the_strict_module_shape() {
    let source = "<style module>.active:global(.loaded) { color: red; }</style>";
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).expect("SFC parses");
    let shape = css_module_block_shape(&descriptor.styles[0], false).expect("known local exports");
    assert_eq!(css_module_type(&shape, true), r#"{ "active": string; }"#);
    assert_eq!(
        css_module_type(&shape, false),
        r#"Record<string, string> & { "active": string; }"#
    );
}

#[test]
fn composed_exports_still_need_the_existing_dynamic_fallback() {
    let source =
        r#"<style module>.active:global(.loaded) { composes: other from "./base.css"; }</style>"#;
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).expect("SFC parses");
    assert!(css_module_block_shape(&descriptor.styles[0], false).is_none());
    assert_eq!(
        css_module_block_type(&descriptor.styles[0], true),
        "Record<string, string>"
    );
}
