use super::{StyleNodeKind, StyleSheet};
use crate::css::lang::StyleLanguage::{self, Css, Less, Sass, Scss, Stylus};
use alloc::vec::Vec;
use vize_l0::{Span, String, cstr};

/// One line per node: depth, kind, prelude, and the name and value of
/// declarations and at-rules.
fn outline(sheet: &StyleSheet<'_>) -> Vec<String> {
    let nodes = sheet.nodes();
    nodes
        .iter()
        .map(|node| {
            let mut depth = 0;
            let mut parent = node.parent();
            while let Some(index) = parent {
                depth += 1;
                parent = nodes.get(index).and_then(|node| node.parent());
            }
            let indent = cstr!("{:width$}", "", width = depth * 2);
            let block = if node.block().is_some() { " {}" } else { "" };
            let prelude = sheet.text(node.prelude());
            match sheet.kind(node) {
                StyleNodeKind::Rule => cstr!("{indent}rule {prelude}{block}"),
                StyleNodeKind::AtRule { name } => {
                    cstr!("{indent}at[{}] {prelude}{block}", sheet.text(name))
                }
                StyleNodeKind::Declaration { name, value } => cstr!(
                    "{indent}decl[{}|{}]{block}",
                    sheet.text(name),
                    sheet.text(value)
                ),
                StyleNodeKind::Other => cstr!("{indent}other {prelude}{block}"),
            }
        })
        .collect()
}

fn read(language: StyleLanguage, source: &str) -> Vec<String> {
    let sheet = language.structure(source, 0);
    assert!(sheet.is_balanced() && sheet.is_terminated(), "{source}");
    outline(&sheet)
}

fn comments(language: StyleLanguage, source: &str) -> Vec<(&str, bool)> {
    let sheet = language.structure(source, 0);
    sheet
        .comments()
        .iter()
        .map(|comment| {
            (
                source
                    .get(comment.span().start as usize..comment.span().end as usize)
                    .unwrap_or_default(),
                comment.is_line(),
            )
        })
        .collect()
}

#[test]
fn css_spans_are_file_absolute_and_comments_stay_beside_nodes() {
    let source = ".a /* x */ .b { color: red; --c: { d: e } }\n@media print { .p { margin: 0 } }";
    let sheet = Css.structure(source, 100);
    assert_eq!(
        outline(&sheet),
        [
            "rule .a /* x */ .b {}",
            "  decl[color|red]",
            "  decl[--c|{ d: e }]",
            "at[media] @media print {}",
            "  rule .p {}",
            "    decl[margin|0]",
        ]
    );
    let rule = sheet.nodes().first().copied().expect("rule");
    assert_eq!(rule.prelude(), Span::new(100, 113));
    assert_eq!(rule.block().map(|block| block.start), Some(114));
    assert_eq!(
        rule.span().end,
        100 + source.find('\n').expect("newline") as u32
    );
    assert_eq!(&*sheet.code(rule.prelude()), ".a  .b");
    assert_eq!(sheet.comments().len(), 1);
    // CSS has no line comments; an unquoted URL payload is opaque.
    assert_eq!(
        read(Css, ".u { background: url(//e.test/a.png); } // .x {}"),
        [
            "rule .u {}",
            "  decl[background|url(//e.test/a.png)]",
            "rule // .x {}"
        ]
    );
}

#[test]
fn scss_reads_nested_properties_variables_and_interpolated_names() {
    assert_eq!(
        read(
            Scss,
            "$gap: 4px;\n@use 'sass:math';\n.a {\n  &:hover { color: red }\n  a:focus {}\n  font: { family: serif; }\n  #{$side}-margin: 1px;\n  @include m;\n  @extend .b\n}"
        ),
        [
            "decl[$gap|4px]",
            "at[use] @use 'sass:math'",
            "rule .a {}",
            "  rule &:hover {}",
            "    decl[color|red]",
            "  rule a:focus {}",
            "  decl[font|] {}",
            "    decl[family|serif]",
            "  decl[#{$side}-margin|1px]",
            "  at[include] @include m",
            "  at[extend] @extend .b",
        ]
    );
    let sheet = Scss.structure(".a-#{$x} {}\n[data-x=\"#{$y}\"] {}\n.c {}", 0);
    let interpolated: Vec<bool> = sheet
        .nodes()
        .iter()
        .map(|node| node.is_interpolated())
        .collect();
    assert_eq!(interpolated, [true, false, false]);
    assert_eq!(
        comments(Scss, "// a\n.u { b: url(http://e.test/x); /* c */ }"),
        [("// a", true), ("/* c */", false)]
    );
}

#[test]
fn less_reads_variables_detached_rulesets_and_mixin_calls() {
    assert_eq!(
        read(
            Less,
            "@gap: 4px;\n@detached: { .d {} };\n.m(@a; @b) { width: @a }\n.c { .m(1; 2); &:extend(.a); @{p}-top: 0; }\n.pre-@{name} {}\n@media screen { .x {} }"
        ),
        [
            "decl[@gap|4px]",
            "decl[@detached|] {}",
            "  rule .d {}",
            "rule .m(@a; @b) {}",
            "  decl[width|@a]",
            "rule .c {}",
            "  other .m(1; 2)",
            "  other &:extend(.a)",
            "  decl[@{p}-top|0]",
            "rule .pre-@{name} {}",
            "at[media] @media screen {}",
            "  rule .x {}",
        ]
    );
}

#[test]
fn sass_reads_indented_blocks_shorthands_and_indented_comments() {
    let source = ".card\n  &__title\n    color: red\n  font:\n    family: serif\n=m($x)\n  width: $x\n+m(1)\n// a silent comment\n   that continues\n/* a loud comment\n   without a close\n.a,\n.b\n  @include n\n";
    assert_eq!(
        read(Sass, source),
        [
            "rule .card {}",
            "  rule &__title {}",
            "    decl[color|red]",
            "  decl[font|] {}",
            "    decl[family|serif]",
            "at[=] =m($x) {}",
            "  decl[width|$x]",
            "at[+] +m(1)",
            "rule .a,\n.b {}",
            "  at[include] @include n",
        ]
    );
    assert_eq!(
        comments(Sass, source),
        [
            ("// a silent comment\n   that continues", true),
            ("/* a loud comment\n   without a close", false),
        ]
    );
    let sheet = Sass.structure(".a\n  b: c\n.d\n", 0);
    let block = sheet.nodes().first().and_then(|node| node.block());
    assert_eq!(block, Some(Span::new(5, 9)));
}

#[test]
fn stylus_reads_colonless_declarations_and_its_own_statements() {
    assert_eq!(
        read(
            Stylus,
            "$gap = 4px\nborder-radius(n)\n  -webkit-border-radius n\n.list\n  for i in 1..3\n    .item-{i}\n      width 10px\n  color: red\n  border-radius(5px)\n  +cache()\n    display block\n/* multi\n   line */\n.b\n  margin 0\n"
        ),
        [
            "other $gap = 4px",
            "other border-radius(n) {}",
            "  decl[-webkit-border-radius|n]",
            "rule .list {}",
            "  other for i in 1..3 {}",
            "    rule .item-{i} {}",
            "      decl[width|10px]",
            "  decl[color|red]",
            "  other border-radius(5px)",
            "  other +cache() {}",
            "    decl[display|block]",
            "rule .b {}",
            "  decl[margin|0]",
        ]
    );
    assert_eq!(
        read(Stylus, ".brace {\n  &-{x} {\n    color red\n  }\n}\n"),
        ["rule .brace {}", "  rule &-{x} {}", "    decl[color|red]"]
    );
}

#[test]
fn unbalanced_and_unterminated_sources_are_flagged_and_still_read() {
    let open = Scss.structure(".a { .b {", 0);
    assert!(!open.is_balanced() && open.is_terminated());
    assert_eq!(open.nodes().len(), 2);
    assert_eq!(open.nodes().first().map(|node| node.span().end), Some(9));
    let stray = Scss.structure(".a {} }", 0);
    assert!(!stray.is_balanced());
    for source in [".a { /* x", ".a { content: 'x", ".a-#{$x {}"] {
        assert!(!Scss.structure(source, 0).is_terminated(), "{source}");
    }
}
