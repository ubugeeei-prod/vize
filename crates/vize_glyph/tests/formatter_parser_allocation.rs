//! Preserve shared parser safety through the formatter's transitive dependency.

use oxc_allocator::Allocator;
use oxc_formatter::parse_for_format;
use oxc_span::SourceType;

#[test]
fn malformed_type_arguments_stay_bounded_in_the_formatter_parse_path() {
    for source in [
        include_str!(
            "../../../tests/fuzz/regressions/l1_program/type-argument-backtracking.tsx.input"
        ),
        include_str!(
            "../../../tests/fuzz/regressions/l1_program/type-argument-slow-unit.tsx.input"
        ),
        include_str!(
            "../../../tests/fuzz/regressions/l1_program/type-argument-computed-key.ts.input"
        ),
    ] {
        let allocator = Allocator::default();
        let parsed = parse_for_format(&allocator, source, SourceType::tsx());
        assert!(!parsed.diagnostics.is_empty());
        assert_eq!(parsed.program.source_text, source);
        assert!(
            allocator.used_bytes() < 1024 * 1024,
            "formatter parser allocated {} bytes",
            allocator.used_bytes()
        );
    }
}

#[test]
fn valid_formatter_generics_keep_the_original_source() {
    let source = "export const value = f<Array<{ name: string }>>();";
    let allocator = Allocator::default();
    let parsed = parse_for_format(&allocator, source, SourceType::ts());
    assert!(parsed.diagnostics.is_empty());
    assert_eq!(parsed.program.source_text, source);
}

#[test]
fn l1_formatter_profile_is_the_parse_the_printer_requires() {
    use vize_l1::embed::syntax::format::{FORMAT_PARSE_OPTIONS, format_source_type};
    assert_eq!(FORMAT_PARSE_OPTIONS, oxc_formatter::format_parse_options());
    for source in [
        "export const value = f<Array<{ name: string }>>();",
        "const view = <a>{b}</a>;",
        "const = ;",
    ] {
        for source_type in [
            SourceType::mjs(),
            SourceType::cjs(),
            SourceType::jsx(),
            SourceType::ts(),
            SourceType::tsx(),
            SourceType::d_ts(),
        ] {
            let allocator = Allocator::default();
            let stock = parse_for_format(&allocator, source, source_type);
            let arena = vize_l0::Allocator::default();
            let profiled =
                vize_l1::embed::syntax::parse_program_for_format(&arena, source, source_type);
            assert_eq!(
                stock.program.source_type.is_jsx(),
                format_source_type(source_type).is_jsx()
            );
            assert_eq!(profiled.program.source_type, stock.program.source_type);
            assert_eq!(profiled.diagnostics.len(), stock.diagnostics.len());
        }
    }
}
