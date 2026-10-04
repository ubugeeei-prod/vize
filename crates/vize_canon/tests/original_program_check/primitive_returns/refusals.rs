use super::{
    Allocator, Lang, OriginalProgramError, Path, ProjectionError, block_on, file, project,
    real_bridge,
};

#[test]
fn unsupported_original_returns_never_reach_a_checker_or_rewrite_source() {
    let arena = Allocator::default();
    let root = tempfile::TempDir::new().unwrap();
    let bridge = project(root.path(), true);
    for source in [
        "function f(value:number):number|string{return value;}f(1);",
        "function f(value:number):number[]{return value;}f(1);",
        "function f(value:number):any{return value;}f(1);",
        "async function f(value:number):number{return value;}f(1);",
        "function f(value:number):number{return missing;}f(1);",
    ] {
        let original = file(&arena, source, Lang::Ts);
        assert!(!original.is_complete());
        let path = root.path().join("OriginalReturn.ts");
        std::fs::write(&path, source).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_program(&original, &path)),
            Err(OriginalProgramError::Projection(
                ProjectionError::IncompleteFile
            ))
        ));
        assert!(core::ptr::eq(original.artifact().source(), source));
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    }
}

#[test]
fn complete_original_return_keeps_authoritative_path_source_and_module_goal_refusals() {
    let arena = Allocator::default();
    let source = "function f(value:number):number{return value;}f(1);";
    let original = file(&arena, source, Lang::Ts);
    assert!(original.is_complete(), "{:?}", original.issues());
    let root = tempfile::TempDir::new().unwrap();
    let bridge = project(root.path(), true);
    let path = root.path().join("OriginalReturn.ts");
    let changed = "function f(value:number):number{return value;}f(2);";
    std::fs::write(&path, changed).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &path)),
        Err(OriginalProgramError::SourceChanged)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), changed.as_bytes());
    std::fs::write(&path, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, Path::new("OriginalReturn.ts"))),
        Err(OriginalProgramError::InvalidSourcePath)
    ));
    let wrong = root.path().join("OriginalReturn.js");
    std::fs::write(&wrong, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &wrong)),
        Err(OriginalProgramError::SourceKindMismatch)
    ));
    let outside = tempfile::TempDir::new().unwrap();
    let foreign = outside.path().join("OriginalReturn.ts");
    std::fs::write(&foreign, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &foreign)),
        Err(OriginalProgramError::OutsideProject)
    ));
    drop(project(root.path(), false));
    let bridge = real_bridge(root.path());
    block_on(bridge.spawn()).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &path)),
        Err(OriginalProgramError::UnsupportedModuleGoal)
    ));
    block_on(bridge.shutdown()).unwrap();
    assert!(core::ptr::eq(original.artifact().source(), source));
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
}
