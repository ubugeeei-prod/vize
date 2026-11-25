//! Both scoped CSS scanners end a quoted string where CSS Syntax ends it: at
//! the next unescaped code point equal to the opening quote. A scalar whose
//! low byte equals a quote (U+0122 `Ģ`, U+0127 `ħ`) is string content, and
//! an escaped backslash does not escape the quote after it.

use vize_carton::Allocator;

use super::{CssCompileOptions, apply_scoped_css, compile_css};

const SCOPE: &str = "data-v-1";

/// Each stylesheet keeps a string open across a selector if the scanner ends
/// it in the wrong place; the expected text scopes `.b`.
const CASES: [(&str, &str); 5] = [
    (
        ".a { content: \"\u{122}\"; }\n.b { color: red; }",
        ".a[data-v-1]{ content: \"\u{122}\"; }.b[data-v-1]{ color: red; }",
    ),
    (
        ".a { content: '\u{127}'; }\n.b { color: red; }",
        ".a[data-v-1]{ content: '\u{127}'; }.b[data-v-1]{ color: red; }",
    ),
    (
        ".a { content: \"\\\\\"; }\n.b { color: red; }",
        ".a[data-v-1]{ content: \"\\\\\"; }.b[data-v-1]{ color: red; }",
    ),
    (
        ".a { content: \"x\\\"\u{122}\\\\\"; }\n.b { color: red; }",
        ".a[data-v-1]{ content: \"x\\\"\u{122}\\\\\"; }.b[data-v-1]{ color: red; }",
    ),
    (
        ".a { content: \"\\\\\\\"\"; }\n.b { color: red; }",
        ".a[data-v-1]{ content: \"\\\\\\\"\"; }.b[data-v-1]{ color: red; }",
    ),
];

#[test]
fn the_compiler_scanner_ends_strings_at_their_unescaped_quote() {
    let bump = Allocator::new();
    for (css, expected) in CASES {
        assert_eq!(apply_scoped_css(&bump, css, SCOPE), expected, "{css:?}");
    }
}

#[test]
fn the_style_block_scanner_ends_strings_at_their_unescaped_quote() {
    for (css, expected) in CASES {
        assert_eq!(
            crate::style::apply_scoped_css(css, SCOPE),
            expected,
            "{css:?}"
        );
    }
}

#[test]
fn compiled_scoped_css_scopes_the_rule_after_a_quote_low_byte() {
    let options = CssCompileOptions {
        scoped: true,
        scope_id: Some(SCOPE.into()),
        minify: true,
        ..Default::default()
    };
    let result = compile_css(".a{content:\"\u{122}\"}.b{color:red}", &options);
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(
        result.code,
        ".a[data-v-1]{content:\"\u{122}\"}.b[data-v-1]{color:red}"
    );
}
