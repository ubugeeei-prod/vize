use vize_l1::css::lang::StyleLanguage;

use super::super::{CssLinter, NoImportant};

/// The `!important` flags `css/no-important` reports in a block at offset 10.
fn flags(language: StyleLanguage, source: &str) -> Vec<String> {
    let mut linter = CssLinter::new();
    linter.add_style_rule(Box::new(NoImportant));
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
}

#[test]
fn an_unbalanced_block_has_no_structure_findings() {
    assert_eq!(
        flags(StyleLanguage::Scss, ".a { color: red !important;"),
        Vec::<String>::new()
    );
}
