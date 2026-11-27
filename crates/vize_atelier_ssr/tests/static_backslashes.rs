//! Authored backslashes are text. A static part is copied into a JavaScript
//! template literal, where a bare backslash would start an escape sequence
//! (`\n` renders a newline, `\x` fails to parse) and would cancel the escape
//! of a backtick or `${` that follows it.

use vize_atelier_ssr::{Allocator, compile_ssr};

/// Compile `source` and require its render body to be one `_push` of exactly
/// `template_literal`.
fn assert_pushed(source: &str, template_literal: &str) {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr(&allocator, source);
    assert!(errors.is_empty(), "{errors:?}");
    let expected = [
        "function ssrRender(_ctx, _push, _parent, _attrs) {\n  _push(`",
        template_literal,
        "`)\n}\n",
    ]
    .concat();
    assert_eq!(result.code.as_str(), expected.as_str());
}

#[test]
fn text_backslashes_stay_literal() {
    assert_pushed(
        r"<p>C:\new\table \u0041 \x41</p>",
        r"<p${_ssrRenderAttrs(_attrs)}>C:\\new\\table \\u0041 \\x41</p>",
    );
}

#[test]
fn a_backslash_does_not_cancel_the_escape_that_follows_it() {
    for source in ["<p>\\`</p>", "<p>&#92;&#96;</p>"] {
        assert_pushed(source, r"<p${_ssrRenderAttrs(_attrs)}>\\\`</p>");
    }
    assert_pushed(
        r"<p v-pre>\${a} {{ b }}</p>",
        r"<p${_ssrRenderAttrs(_attrs)}>\\\${a} {{ b }}</p>",
    );
}

#[test]
fn attribute_backslashes_stay_literal() {
    assert_pushed(
        r#"<p><i title="a\b\`"></i></p>"#,
        r#"<p${_ssrRenderAttrs(_attrs)}><i title="a\\b\\\`"></i></p>"#,
    );
}
