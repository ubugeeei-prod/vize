//! Regression coverage for dotted named slots in the Vapor compiler.
//!
//! Kept separate from `tests.rs` so that already large file does not grow past
//! the source-file-length limit; the small assertion helpers mirror the ones
//! defined there.

use super::compile_vapor;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::Bump;

fn normalize_code(code: &str) -> String {
    code.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_parses_as_module(code: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        code,
        SourceType::default()
            .with_module(true)
            .with_typescript(true),
    )
    .parse();

    assert!(
        parsed.errors.is_empty(),
        "generated code should parse, got: {:?}\n\n{}",
        parsed.errors,
        code
    );
}

#[test]
fn test_compile_component_preserves_dotted_slot_names() {
    let allocator = Bump::new();
    let result = compile_vapor(
        &allocator,
        r#"<MyComponent><template #item.alpha="{ value }">{{ value }}</template><template #item.beta="{ value }">{{ value }}</template><template #item.gamma="{ item }">{{ item }}</template></MyComponent>"#,
        Default::default(),
    );

    assert!(
        result.error_messages.is_empty(),
        "Expected no errors: {:?}",
        result.error_messages
    );

    let code = normalize_code(&result.code);
    assert_parses_as_module(&code);
    assert!(
        code.contains(r#""item.alpha": (_slotProps0) => {"#),
        "{}",
        code
    );
    assert!(code.contains(r#"_slotProps0.value"#), "{}", code);
    assert!(
        code.contains(r#""item.beta": (_slotProps1) => {"#),
        "{}",
        code
    );
    assert!(code.contains(r#"_slotProps1.value"#), "{}", code);
    assert!(
        code.contains(r#""item.gamma": (_slotProps2) => {"#),
        "{}",
        code
    );
    assert!(code.contains(r#"_slotProps2.item"#), "{}", code);
}
