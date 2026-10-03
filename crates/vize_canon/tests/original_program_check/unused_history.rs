//! Batch's complete typed function never grants incomplete native authority.

use super::*;

#[test]
fn historical_unused_typed_parameter_keeps_original_native_refusal() {
    let source = include_str!(
        "../../../../tests/_fixtures/differential/typechecker/authored-unused-symbols/helper.ts.txt"
    );
    let arena = Allocator::default();
    let original = file(&arena, source, Lang::Ts);
    assert!(!original.is_complete());
    assert_eq!(original.artifact().source(), source);
    assert!(!original.issues().is_empty());
    let root = tempfile::TempDir::new().unwrap();
    let path = root.path().join("helper.ts");
    std::fs::write(&path, source).unwrap();
    let bridge = project(root.path(), true);
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &path)),
        Err(OriginalProgramError::Projection(
            ProjectionError::IncompleteFile
        ))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
}
