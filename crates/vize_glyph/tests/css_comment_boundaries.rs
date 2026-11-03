use lightningcss::stylesheet::{ParserOptions, StyleSheet};
use vize_glyph::{FormatOptions, format_sfc, format_style};

#[test]
fn comments_inside_rule_preludes_keep_the_authored_css() {
    let options = FormatOptions::default();
    for source in [
        ".foo/* selector */.bar { color: red; }",
        ".foo /* descendant */ .bar { color: red; }",
        "@media /* query */ (min-width: 1px) { .box { color: red; } }",
        "@import /* import */ url(\"theme.css\");",
        ".box:is(.a, /* alternative */ .b) { color: red; }",
    ] {
        StyleSheet::parse(source, ParserOptions::default()).expect("valid authored CSS");
        let formatted = format_style(source, &options).unwrap();
        assert_eq!(formatted.as_str(), source);
        assert_eq!(format_style(&formatted, &options).unwrap(), formatted);
    }
}

#[test]
fn sfc_style_comments_keep_selector_adjacency() {
    let source = "<style scoped>\n.foo/* selector */.bar { color: red; }\n</style>\n";
    let options = FormatOptions::default();
    let formatted = format_sfc(source, &options).unwrap().code;
    assert_eq!(formatted.as_str(), source);
    assert_eq!(format_sfc(&formatted, &options).unwrap().code, formatted);
}

#[test]
fn comments_between_complete_rules_still_allow_formatting() {
    let options = FormatOptions::default();
    for source in [
        ".a{color:red}\n/* between */\n.b{color:blue}",
        "@import url(\"theme.css\");\n/* between */\n.b{color:blue}",
    ] {
        let formatted = format_style(source, &options).unwrap();
        assert!(
            formatted.contains("/* between */\n.b {\n  color: blue;\n}"),
            "{formatted}"
        );
        assert_eq!(format_style(&formatted, &options).unwrap(), formatted);
    }
}
