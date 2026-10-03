//! Shared parser safety for malformed TSX generic arguments (#7421, #7444).

use vize_atelier_jsx::{JsxLang, parse_module};
use vize_l0::Allocator;

#[test]
fn malformed_nested_type_arguments_have_bounded_arena_allocation() {
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
        let allocator = Allocator::new();
        let parsed = parse_module(allocator.as_oxc(), source, JsxLang::Tsx);
        assert!(
            parsed.has_errors(),
            "malformed input must remain a diagnostic"
        );
        assert_eq!(parsed.program.source_text, source);
        assert!(
            allocator.allocated_bytes() < 1024 * 1024,
            "a sub-3KiB malformed source allocated {} arena bytes",
            allocator.allocated_bytes()
        );
    }
}

#[test]
fn valid_nested_generics_and_relational_expressions_still_parse() {
    let source = r#"
        const slots = defineSlots<{
            default(ps?: { value: Array<Record<string, number>> }): unknown;
        }>();
        const nested = f<T>(g<U>());
        const callback = f<(x: number) => string>();
        const less = value < other;
        const greater = value >= other;
        const shift = value << other;
        const component = <Child value={nested} />;
    "#;
    let allocator = Allocator::new();
    let parsed = parse_module(allocator.as_oxc(), source, JsxLang::Tsx);
    assert!(!parsed.has_errors(), "{:?}", parsed.diagnostics);
    assert_eq!(parsed.program.source_text, source);
    assert_eq!(parsed.program.body.len(), 7);
}
