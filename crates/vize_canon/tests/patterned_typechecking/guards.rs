#![expect(clippy::string_slice, reason = "tests assert by panicking")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
use crate::{BatchTypeChecker, BatchTypeCheckerTrait, SfcBlockType, project, write};

#[test]
fn guard_line_comments_keep_narrowing_and_authored_type_errors() {
    let project = project();
    let mut expected = Vec::new();
    for (index, newline) in [
        "\n",
        "\r",
        "\r\n",
        "\u{2028}",
        "\u{2029}",
        "&#10;",
        "&#13;&#10;",
    ]
    .into_iter()
    .enumerate()
    {
        for (case, continuation, method, diagnostic) in [
            ("Valid", "", "toUpperCase", None),
            ("Body", "", "toFixed", Some(("toFixed", 2551))),
            (
                "Guard",
                " && text.missing()",
                "toUpperCase",
                Some(("missing", 2339)),
            ),
        ] {
            let source = vize_s0::cstr!(
                "<script setup lang=\"ts\">\nconst state = undefined as string | undefined;\n</script>\n<template v-match=\"state\">\n<p v-when=\"const text if (text !== undefined // guard ){newline}{continuation})\">{{{{ text.{method}() }}}}</p>\n<p v-when=\"_\"/>\n</template>"
            );
            let name = vize_s0::cstr!("Case{index}{case}.vue");
            write(project.path(), &vize_s0::cstr!("src/{name}"), &source);
            if let Some((needle, code)) = diagnostic {
                let before = &source[..source.find(needle).unwrap()];
                let before = before
                    .replace("\r\n", "\n")
                    .replace(['\r', '\u{2028}', '\u{2029}'], "\n");
                let line = before.bytes().filter(|byte| *byte == b'\n').count();
                let column = before.rsplit('\n').next().unwrap().encode_utf16().count();
                expected.push((name.to_string(), Some(code), line as u32, column as u32));
            }
        }
    }
    let mut checker = BatchTypeChecker::new(project.path()).unwrap();
    checker.set_experimental_patterned_template(true);
    checker.scan_project().unwrap();
    let result = checker.check_project().unwrap();
    assert!(!result.success, "negative fixtures must fail");
    let mut actual = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.block_type, Some(SfcBlockType::Template));
            assert_eq!(diagnostic.severity, 1);
            (
                diagnostic
                    .file
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                diagnostic.code,
                diagnostic.line,
                diagnostic.column,
            )
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected, "{:#?}", result.diagnostics);
}
