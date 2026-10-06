//! Preserve explicit JSX/TSX script boundaries without changing regex contents.
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use super::parse_sfc;
use crate::sfc::types::SfcParseOptions;

const TSX: &str = include_str!(
    "../../../../../tests/_fixtures/differential/sfc/script-jsx-boundary-7982/tsx.vue.txt"
);
const JSX: &str = include_str!(
    "../../../../../tests/_fixtures/differential/sfc/script-jsx-boundary-7982/jsx.vue.txt"
);

fn assert_setup(source: &str, language: &str, expected: &str) {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    assert_eq!(descriptor.source, source);
    assert!(descriptor.script.is_none());
    let script = descriptor.script_setup.unwrap();
    let start = source.find('>').unwrap() + 1;
    let end = source.rfind("</script>").unwrap();
    assert_eq!(script.content, expected);
    assert_eq!(&source[start..end], expected);
    assert_eq!((script.loc.start, script.loc.end), (start, end));
    assert_eq!((script.loc.tag_start, script.loc.tag_end), (0, end + 9));
    let location = |offset| {
        let prefix = &source[..offset];
        (
            prefix.bytes().filter(|b| *b == b'\n').count() + 1,
            prefix.rfind('\n').map_or(offset + 1, |last| offset - last),
        )
    };
    assert_eq!(
        (script.loc.start_line, script.loc.start_column),
        location(start)
    );
    assert_eq!((script.loc.end_line, script.loc.end_column), location(end));
    assert_eq!(script.lang.as_deref(), Some(language));
    assert!(script.setup && script.src.is_none() && script.bindings.is_none());
    assert_eq!(script.attrs.len(), 2);
    assert_eq!(script.attrs.get("lang").map(|v| v.as_ref()), Some(language));
    assert_eq!(script.attrs.get("setup").map(|v| v.as_ref()), Some(""));
    assert_eq!(descriptor.template.unwrap().content, "");
    assert!(descriptor.styles.is_empty() && descriptor.custom_blocks.is_empty());
    assert!(descriptor.css_vars.is_empty());
}

#[test]
fn original_paired_jsx_closing_elements_preserve_complete_script_locations() {
    for (language, source) in [("tsx", TSX), ("jsx", JSX)] {
        assert_setup(
            source,
            language,
            "const value = <p>{window.innerWidth}</p>;",
        );
    }
    for language in ["tsx", "jsx"] {
        for body in [
            "const value = <Outer><p>{window.innerWidth}</p></Outer>;",
            "const value = <><Foo.Bar>{window.innerWidth}</Foo.Bar></>;",
            "\r\nconst value = <p>{window.innerWidth}</p>;\r\n",
        ] {
            let source = format!("<script setup lang=\"{language}\">{body}</script><template />");
            assert_setup(&source, language, body);
        }
    }
}

#[test]
fn genuine_comparison_regex_and_character_classes_keep_their_whole_authored_bytes() {
    for language in ["tsx", "jsx", "ts", "js"] {
        for body in [
            "const ok = value </p>[</script>]/.test(text);",
            "const pattern = /[</script>\\/]/giu;",
            "const text = `</script>`; const pattern = /script/;",
        ] {
            let source = format!("<script setup lang=\"{language}\">{body}</script><template />");
            assert_setup(&source, language, body);
        }
    }
}
