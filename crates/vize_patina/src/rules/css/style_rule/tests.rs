use vize_l1::css::lang::StyleLanguage;

use super::super::{CssLinter, NoIdSelectors, NoImportant, StyleRule};

/// The `!important` flags `css/no-important` reports in a block at offset 10.
fn flags(language: StyleLanguage, source: &str) -> Vec<String> {
    findings(Box::new(NoImportant), language, source)
}

/// The text of each finding of `rule` in a block at offset 10.
fn findings(rule: Box<dyn StyleRule>, language: StyleLanguage, source: &str) -> Vec<String> {
    let mut linter = CssLinter::new();
    linter.add_style_rule(rule);
    linter
        .lint_block(source, 10, language, true)
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let start = diagnostic.start as usize - 10;
            let end = diagnostic.end as usize - 10;
            source.get(start..end).unwrap().to_owned()
        })
        .collect()
}

#[test]
fn css_flags_are_terminal_top_level_value_tokens() {
    let css = StyleLanguage::Css;
    assert_eq!(
        flags(css, ".a { width: 1px !/**/IMPORTANT; }"),
        ["!/**/IMPORTANT"]
    );
    // A function before the flag is consumed whole.
    assert_eq!(
        flags(css, "a{opacity:min(10%,90%)!important}"),
        ["!important"]
    );
    assert_eq!(
        flags(css, ".a { &:hover { color: red !important; } }"),
        ["!important"]
    );
    // Quoted, commented, selector and nested custom-property text is not a flag.
    let opaque = r#"/* !important */
.a[data-value="!important"] {
  content: "quote\" !important";
  background: url("x !important");
  --raw: { note: '!important'; actual: red !important };
  --last: next ! important;
}"#;
    assert_eq!(flags(css, opaque), ["! important"]);
    // A flag that is not last is not an important flag.
    assert_eq!(
        flags(css, ".a { color: red !important blue; }"),
        Vec::<String>::new()
    );
}

#[test]
fn preprocessor_blocks_report_flags_at_authored_bytes() {
    let scss = ".a {\n  color: red !important; // !important\n  $gap: 1px !important !default;\n  .b { margin: 0 !important }\n}";
    assert_eq!(
        flags(StyleLanguage::Scss, scss),
        ["!important", "!important", "!important"]
    );
    let less = ".a { .mixin() !important; color: red !important; }";
    assert_eq!(flags(StyleLanguage::Less, less), ["!important"]);
    let sass = ".a\n  color: red !important\n  // margin: 0 !important\n";
    assert_eq!(flags(StyleLanguage::Sass, sass), ["!important"]);
    let stylus = ".a\n  color red !important\n";
    assert_eq!(flags(StyleLanguage::Stylus, stylus), ["!important"]);
    let braced = ".e { margin: 0 !important; }";
    assert_eq!(flags(StyleLanguage::Stylus, braced), ["!important"]);
    // A `//` comment inside a value cannot open a block that hides the flag.
    let commented = ".a { color: red // explanation (\n !important; }";
    assert_eq!(flags(StyleLanguage::Scss, commented), ["!important"]);
}

#[test]
fn an_unbalanced_block_has_no_structure_findings() {
    assert_eq!(
        flags(StyleLanguage::Scss, ".a { color: red !important;"),
        Vec::<String>::new()
    );
}

fn ids(language: StyleLanguage, source: &str) -> Vec<String> {
    findings(Box::new(NoIdSelectors), language, source)
}

#[test]
fn id_selectors_are_top_level_selector_tokens() {
    let css = StyleLanguage::Css;
    // The key compound of each selector, right to left, as before.
    assert_eq!(
        ids(css, "#a, #b .c, .d > #e#f { color: #fff; }"),
        ["#a", "#f", "#e"]
    );
    // A functional pseudo-class is consumed whole; the ID after it is the rule's.
    assert_eq!(
        ids(css, ".a:not(#b) #c, [x] #d { color: red; }"),
        ["#c", "#d"]
    );
    // Pseudo-class arguments, attribute values, comments and colors are not.
    assert_eq!(
        ids(
            css,
            ".a:not(#b)[href=\"#c\"] /* #d */ { background: #eee; }"
        ),
        Vec::<String>::new()
    );
    assert_eq!(
        ids(css, "@media (width > 1px) { #m { color: red; } }"),
        ["#m"]
    );
    assert_eq!(
        ids(css, "@keyframes k { from { opacity: 0; } }"),
        Vec::<String>::new()
    );
    // As before: rules under at-rules the lightningcss lane skipped are not checked.
    assert_eq!(
        ids(css, "@container (width > 1px) { #c { color: red; } }"),
        Vec::<String>::new()
    );
    assert_eq!(
        ids(
            css,
            "@supports (display: grid) { @layer a { #s { color: red; } } }"
        ),
        ["#s"]
    );
}

#[test]
fn preprocessor_selectors_report_ids_as_written() {
    let scss = ".card {\n  #title { color: red; }\n  &#{$x} { color: blue; }\n  // #note\n}";
    assert_eq!(ids(StyleLanguage::Scss, scss), ["#title"]);
    let less = "#ns { .mixin() { color: red; } }\n.a { #ns.mixin(); }";
    assert_eq!(ids(StyleLanguage::Less, less), ["#ns"]);
    let sass = "#main\n  color: #fff\n  .x\n    width: 1px\n";
    assert_eq!(ids(StyleLanguage::Sass, sass), ["#main"]);
    let stylus = "#main\n  color #fff\n";
    assert_eq!(ids(StyleLanguage::Stylus, stylus), ["#main"]);
}
