use super::{TsrxCompileOptionsNapi, TsrxDiagnosticNapi, compile_tsrx_impl};
use vize_l4::lang::tsrx::{CompileOptions, Target, compile};

const BADGE: &str = "export function Badge({ text }: { text: string }) @{\n  <span class=\"badge\">{text}</span>\n}\n";

fn options(target: Option<&str>, erase_types: bool) -> TsrxCompileOptionsNapi {
    TsrxCompileOptionsNapi {
        filename: Some("/src/Badge.tsrx".to_owned()),
        target: target.map(str::to_owned),
        source_map: None,
        erase_types: Some(erase_types),
    }
}

fn diagnostic(
    code: &str,
    message: &str,
    at: (u32, u32),
    line: u32,
    column: u32,
) -> TsrxDiagnosticNapi {
    TsrxDiagnosticNapi {
        code: code.to_owned(),
        message: message.to_owned(),
        start: at.0,
        end: at.1,
        line,
        column,
    }
}

#[test]
fn a_compiled_module_is_the_compile_entry_output() {
    for (target, erase_types) in [(None, false), (Some("dom"), false), (Some("dom"), true)] {
        let result = compile_tsrx_impl(BADGE, options(target, erase_types));
        let expected = compile(
            BADGE,
            &CompileOptions {
                filename: "/src/Badge.tsrx",
                target: Target::Dom,
                source_map: false,
                erase_types,
            },
        )
        .unwrap();
        assert_eq!(result.errors, []);
        assert_eq!(result.code.as_deref(), Some(expected.code.as_str()));
        assert_eq!(result.map, None);
    }
}

#[test]
fn a_refused_module_has_every_diagnostic_and_no_code() {
    let source = include_str!("../../../../../tests/_fixtures/tsrx/errors/code-block-return.tsrx");
    let result = compile_tsrx_impl(source, TsrxCompileOptionsNapi::default());
    assert_eq!(result.code, None);
    assert_eq!(
        result.errors,
        [diagnostic(
            "TSRX2001",
            "Return statements are not allowed inside TSRX templates. Move the return before the TSRX return value, or use conditional rendering instead.",
            (62, 74),
            2,
            18,
        )]
    );

    let two = "export function A({ a }: { a: number }) @{\n  @switch (a) {\n    @case 1: {\n      <p>a</p>\n    }\n  }\n}\n\nexport function B({ b }: { b: number }) @{\n  <div>\n    @try {\n      <p>{b}</p>\n    } @catch (error) {\n      <p>failed</p>\n    }\n  </div>\n}\n";
    let result = compile_tsrx_impl(two, TsrxCompileOptionsNapi::default());
    assert_eq!(result.code, None);
    let lines: Vec<_> = result
        .errors
        .iter()
        .map(|error| (error.code.as_str(), error.line, error.column))
        .collect();
    assert_eq!(lines, [("unsupported", 2, 2), ("unsupported", 11, 4)]);
}

#[test]
fn columns_count_utf16_units_of_the_line() {
    // `é` is two UTF-8 bytes and one UTF-16 unit; `😀` is four and two.
    let source = "const label = \"é😀\"; export const app = () => <p>hi</p>;\n";
    let result = compile_tsrx_impl(source, TsrxCompileOptionsNapi::default());
    let at = u32::try_from(source.find("<p>").unwrap()).unwrap();
    assert_eq!(result.code, None);
    assert_eq!(
        result.errors,
        [diagnostic(
            "unsupported",
            "OutsideComponent is not emitted for the DOM target yet",
            (at, result.errors[0].end),
            1,
            at - 3,
        )]
    );
}

#[test]
fn targets_without_emission_and_unknown_targets_are_refused() {
    let end = u32::try_from(BADGE.len()).unwrap();
    for (target, message) in [
        ("ssr", "Ssr emission of TSRX modules is not supported yet"),
        (
            "vapor",
            "Vapor emission of TSRX modules is not supported yet",
        ),
    ] {
        let result = compile_tsrx_impl(BADGE, options(Some(target), false));
        assert_eq!(result.code, None);
        assert_eq!(
            result.errors,
            [diagnostic("unsupported", message, (0, end), 1, 0)]
        );
    }
    let result = compile_tsrx_impl(BADGE, options(Some("react"), false));
    assert_eq!(result.code, None);
    assert_eq!(
        result.errors,
        [diagnostic(
            "unsupported",
            "unknown TSRX target \"react\"; expected \"dom\", \"ssr\" or \"vapor\"",
            (0, 0),
            1,
            0,
        )]
    );
}
