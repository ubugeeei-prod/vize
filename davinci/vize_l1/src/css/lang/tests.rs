use super::StyleLanguage::{self, Css, Less, Sass, Scss, Stylus};
use alloc::vec::Vec;

fn names(language: StyleLanguage, source: &str) -> Vec<vize_l0::String> {
    language
        .authored_classes(source)
        .expect("readable source")
        .into_names()
        .into_iter()
        .collect()
}

fn complete(language: StyleLanguage, source: &str) -> bool {
    language
        .authored_classes(source)
        .expect("readable source")
        .is_complete()
}

#[test]
fn lang_values_name_one_dialect_each() {
    for (lang, expected) in [
        (None, Some(Css)),
        (Some("css"), Some(Css)),
        (Some("postcss"), Some(Css)),
        (Some("scss"), Some(Scss)),
        (Some("SCSS"), Some(Scss)),
        (Some("sass"), Some(Sass)),
        (Some("less"), Some(Less)),
        (Some("stylus"), Some(Stylus)),
        (Some("styl"), Some(Stylus)),
        (Some(""), None),
    ] {
        assert_eq!(StyleLanguage::from_lang(lang), expected, "{lang:?}");
    }
}

#[test]
fn css_is_complete_unless_interpolation_is_left_in_it() {
    assert_eq!(
        names(Css, ".a, .b:hover { }\n@media print { .c {} }"),
        ["a", "b", "c"]
    );
    assert!(complete(Css, ".a {}\n@import './base.css';"));
    assert!(!complete(Css, ".a-#{$x} {}"));
}

#[test]
fn nested_parent_references_resolve_against_every_parent() {
    let source =
        ".card { &__title { color: red; } &--active {} .icon {} &:hover {} }\n.a, .b { &-x {} }";
    assert_eq!(
        names(Scss, source),
        [
            "a",
            "a-x",
            "b",
            "b-x",
            "card",
            "card--active",
            "card__title",
            "icon"
        ]
    );
    assert!(complete(Scss, source));
}

#[test]
fn comments_values_and_nested_properties_add_no_names() {
    assert_eq!(
        names(
            Scss,
            "// .line {}\n/* .block {} */\n$gap: 4px;\n@use 'theme';\n.u {\n  background: url(http://e.test/a.png); // .trailing {}\n  font: { family: serif; }\n  content: '.quoted {';\n  &-v {}\n}"
        ),
        ["u", "u-v"]
    );
}

#[test]
fn at_rules_keep_or_drop_the_enclosing_selector() {
    assert_eq!(
        names(
            Scss,
            "@media (max-width: 1px) { .m { &-n {} } }\n.p { @media print { &-q {} } @include wide { &-r {} } @at-root .s {} }\n@mixin m { .mixed { &-x {} } }\n@each $k in a, b { .each-#{$k} {} }\n.i-#{$name} { &-j {} }\n.ok {}\n:global(.g) .l {}"
        ),
        ["l", "m", "m-n", "ok", "p", "p-q", "p-r", "s"]
    );
}

#[test]
fn less_resolves_parents_and_skips_mixin_definitions() {
    assert_eq!(
        names(
            Less,
            "@gap: 4px;\n.card { &__title {} &:extend(.base); .icon {} }\n.m() { .hidden {} }\n.guarded(@a) when (@a > 1) { .alsoHidden {} }\n#ns { .inner() {} }\n.pre-@{name} { &-x {} }\n@media screen { .m2 { &-n {} } }\n@detached: { .det {} };\n// .commented {}\n.call { .m(); }"
        ),
        ["call", "card", "card__title", "icon", "m2", "m2-n"]
    );
}

#[test]
fn unbalanced_sources_refuse() {
    assert_eq!(Scss.authored_classes(".a { &-b {"), None);
    assert_eq!(Scss.authored_classes(".a {} }"), None);
}

#[test]
fn braced_completeness_tracks_constructs_that_add_unseen_rules() {
    assert!(complete(
        Scss,
        ".a { &-b { color: red; } }\n@media print { .c {} }\n@use 'sass:math';\n$gap: 4px;\n@mixin m { .hidden {} @include n; }\n@function f($x) { @return $x; }\n@keyframes spin { from { opacity: 0 } }\n.d { font: { family: serif; } @extend .a; }"
    ));
    for source in [
        ".a { @include m; }",
        ".a { @include m }",
        ".a { @include bp(md) { &-x {} } }",
        "@use 'theme';",
        "@import 'x';",
        "@each $k in a, b { .e-#{$k} {} }",
        "@if $x { .f {} }",
        ".g-#{$x} {}",
        ".caf\u{e9} {}",
    ] {
        assert!(!complete(Scss, source), "{source}");
    }
    assert!(complete(
        Less,
        ".a { &-b {} }\n.m() { .hidden {} }\n@gap: 4px;\n@detached: { .det {} };\n.c:extend(.a) {}"
    ));
    for source in [
        ".a { .m(); }",
        ".a { .m }",
        ".a { #ns > .m(); }",
        "@detached();",
        "@import 'x';",
        ".pre-@{name} {}",
    ] {
        assert!(!complete(Less, source), "{source}");
    }
}

#[test]
fn sass_resolves_indented_parents_and_skips_mixins() {
    let sass = ".card\n  &__title\n    color: red\n  // .commented\n  .icon\n    color: blue\n=mixin\n  .hidden\n    color: red\n.a, .b\n  &-x\n    color: red\n@media screen\n  .m\n    &-n\n      color: red\n.i-#{$x}\n  &-j\n    color: red\n.np\n  font:\n    family: serif\n  &-z\n    background: url(http://e.test/a.png)\n/* .block\n   comment */\n+wide\n  .content\n    color: red\n";
    assert_eq!(
        names(Sass, sass),
        [
            "a",
            "a-x",
            "b",
            "b-x",
            "card",
            "card__title",
            "content",
            "icon",
            "m",
            "m-n",
            "np",
            "np-z"
        ]
    );
}

#[test]
fn stylus_resolves_indented_parents_and_skips_functions_and_loops() {
    let stylus = ".card\n  &__title\n    color red\n  .icon\n    color blue\nborder-radius(n)\n  -webkit-border-radius n\n.list\n  for i in 1..3\n    .item-{i}\n      width 10px\n  &-x\n    color red\n.a,\n.b\n  &-y\n    color red\n";
    assert_eq!(
        names(Stylus, stylus),
        [
            "a",
            "a-y",
            "b",
            "b-y",
            "card",
            "card__title",
            "icon",
            "list",
            "list-x"
        ]
    );
    assert_eq!(
        names(Stylus, ".brace {\n  &-x {\n    color red\n  }\n}\n"),
        ["brace", "brace-x"]
    );
}

#[test]
fn indented_completeness_tracks_constructs_that_add_unseen_rules() {
    assert!(complete(
        Sass,
        ".a\n  &-b\n    color: red\n  @extend .c\n=m\n  .hidden\n    color: red\n  +n\n@use 'sass:math'\n@media print\n  .c\n    color: red\n"
    ));
    for source in [
        ".a\n  +m\n",
        ".a\n  +bp(md)\n    &-x\n      color: red\n",
        "@import x\n",
        "@each $k in a, b\n  .e-#{$k}\n    color: red\n",
        ".g-#{$x}\n  color: red\n",
    ] {
        assert!(!complete(Sass, source), "{source}");
    }
    assert!(complete(
        Stylus,
        ".a\n  &-b\n    color red\nborder-radius(n)\n  -webkit-border-radius n\n  .hidden\n    color red\n"
    ));
    for source in [
        ".a\n  border-radius(5px)\n",
        "@import 'x'\n",
        ".list\n  for i in 1..3\n    .item-{i}\n      width 10px\n",
        ".a\n  +cache()\n    color red\n",
        ".brace {\n  color red\n}\n",
    ] {
        assert!(!complete(Stylus, source), "{source}");
    }
}

#[test]
fn urls_comments_in_parentheses_and_string_interpolation_read_as_written() {
    // An unquoted URL payload is not a comment opener.
    assert_eq!(
        names(
            Css,
            "@import url(https://e.test/a/*/reset.css);\n/* note */\n.asset{color:red}"
        ),
        ["asset"]
    );
    // `//` inside a map is a comment, so its quote opens no string.
    assert_eq!(
        names(Scss, "$m: (\n  // 'a': f('b),\n  c: d,\n);\n.x {}"),
        ["x"]
    );
    // Interpolation inside an attribute string leaves the selector known.
    for language in [Scss, Sass] {
        let source = match language {
            Scss => "[data-x=\"#{$a}\"] { .b {} }",
            _ => "[data-x=\"#{$a}\"]\n  .b\n    c: d\n",
        };
        assert_eq!(names(language, source), ["b"], "{language:?}");
        assert!(complete(language, source), "{language:?}");
    }
    // Sass comments that start a line cover the deeper lines after them.
    assert_eq!(
        names(Sass, "/* open\n   .hidden\n.shown\n  a: b\n"),
        ["shown"]
    );
}
