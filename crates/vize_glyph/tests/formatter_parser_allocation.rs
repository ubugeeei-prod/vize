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
