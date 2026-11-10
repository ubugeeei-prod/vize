//! A `<script setup>` that does not parse is a compile error carrying the
//! parse diagnostics, as in Vue (#6844). The parser profile follows `lang`.

use super::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};
use vize_carton::cstr;

fn compile(source: &str) -> Result<crate::types::SfcCompileResult, crate::types::SfcError> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    compile_sfc(&descriptor, SfcCompileOptions::default())
}

fn syntax_error(source: &str) -> crate::types::SfcError {
    let Err(error) = compile(source) else {
        panic!("compiled a setup that does not parse:\n{source}");
    };
    assert_eq!(
        error.code.as_deref(),
        Some("SCRIPT_SETUP_SYNTAX"),
        "{error:?}"
    );
    error
}

#[test]
fn an_unclosed_call_fails_with_its_location() {
    let source = "<script setup lang=\"ts\">\nimport { ref } from 'vue'\nconst a = ref(1)\npick(\nfunction pick() {}\n</script>\n<template><div>{{ a }}</div></template>\n";
    let error = syntax_error(source);
    let loc = error
        .loc
        .clone()
        .expect("the error is located in the .vue file");
    // The call is still open at the end of the setup content, where the
    // parser reports it: offset 93, the start of the `</script>` line.
    assert_eq!(
        (
            error.message.as_str(),
            loc.start,
            loc.end,
            loc.start_line,
            loc.start_column
        ),
        (
            "<script setup> does not parse\n  Expected `)` but found `EOF`",
            93,
            93,
            6,
            1
        )
    );
}

#[test]
fn a_truncated_expression_fails() {
    syntax_error(
        "<script setup>\nconst a = 1 +\n</script>\n<template><div>{{ a }}</div></template>\n",
    );
}

#[test]
fn typescript_needs_lang_ts() {
    let body = "const a: number = 1\n</script>\n<template><div>{{ a }}</div></template>\n";
    syntax_error(&cstr!("<script setup>\n{body}"));
    assert!(compile(&cstr!("<script setup lang=\"ts\">\n{body}")).is_ok());
}

#[test]
fn jsx_needs_lang_jsx_or_tsx() {
    let body = "const node = <div />\n</script>\n<template><div>{{ node }}</div></template>\n";
    syntax_error(&cstr!("<script setup>\n{body}"));
    syntax_error(&cstr!("<script setup lang=\"ts\">\n{body}"));
    for lang in ["jsx", "tsx"] {
        let result = compile(&cstr!("<script setup lang=\"{lang}\">\n{body}"));
        assert!(result.is_ok(), "{lang}: {result:?}");
    }
}
