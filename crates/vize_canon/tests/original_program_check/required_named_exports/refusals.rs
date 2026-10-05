use super::{
    Allocator, Lang, OriginalProgramError, Path, ProjectionError, assert_diagnosing_options,
    block_on, expected, file, project, real_bridge,
};

#[test]
fn unsupported_original_named_exports_never_reach_a_checker_or_rewrite_source() {
    let arena = Allocator::default();
    let root = tempfile::TempDir::new().unwrap();
    let bridge = project(root.path(), true);
    for source in [
        "export function f(value?:number){return value;}f(1);",
        "export function f(value?:number):number{return value;}f(1);",
        "export default function f(value:number):number{return value;}",
        "export function f(value:number|string):number{return value;}f(1);",
        "export function f(value:number):number|string{return value;}f(1);",
        "export function f(value:number):number[]{return value;}f(1);",
        "export function f(value:number):any{return value;}f(1);",
        "export async function f(value:number):number{return value;}f(1);",
        "export function f(value:number):number{return missing;}f(1);",
        "export function f(value:number):number{return value;}export type {f};",
    ] {
        let original = file(&arena, source, Lang::Ts);
        assert!(!original.is_complete());
        let path = root.path().join("OriginalNamedExport.ts");
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
fn complete_original_named_export_keeps_authoritative_path_source_kind_and_project_refusals() {
    let arena = Allocator::default();
    let source = "export function f(value:number):number{return value;}f(1);";
    let original = file(&arena, source, Lang::Ts);
    assert!(original.is_complete(), "{:?}", original.issues());
    let root = tempfile::TempDir::new().unwrap();
    let bridge = project(root.path(), true);
    let path = root.path().join("OriginalNamedExport.ts");
    let changed = "export function f(value:number):number{return value;}f(2);";
    std::fs::write(&path, changed).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &path)),
        Err(OriginalProgramError::SourceChanged)
    ));
    assert_eq!(std::fs::read(&path).unwrap(), changed.as_bytes());
    std::fs::write(&path, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, Path::new("OriginalNamedExport.ts"))),
        Err(OriginalProgramError::InvalidSourcePath)
    ));
    let wrong = root.path().join("OriginalNamedExport.js");
    std::fs::write(&wrong, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &wrong)),
        Err(OriginalProgramError::SourceKindMismatch)
    ));
    let outside = tempfile::TempDir::new().unwrap();
    let foreign = outside.path().join("OriginalNamedExport.ts");
    std::fs::write(&foreign, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_original_program(&original, &foreign)),
        Err(OriginalProgramError::OutsideProject)
    ));
    assert!(core::ptr::eq(original.artifact().source(), source));
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
}

#[test]
fn actual_named_export_supplies_module_goal_without_a_fabricated_force_requirement() {
    let arena = Allocator::default();
    let pack: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../davinci/vize_l4/tests/fixtures/required-named-export-program-checker.json"
    ))
    .unwrap();
    let case = &pack["cases"][0];
    assert_eq!(case["id"], "named-export-valid");
    assert!(case["diagnostics"].as_array().unwrap().is_empty());
    let source = case["source"].as_str().unwrap();
    let original = file(&arena, source, Lang::Ts);
    assert!(original.is_complete());
    assert_eq!(original.exports().len(), 1);
    let root = tempfile::TempDir::new().unwrap();
    drop(project(root.path(), false));
    let config = root.path().join("tsconfig.json");
    let config_bytes = std::fs::read(&config).unwrap();
    let path = root.path().join("OriginalNamedExport.ts");
    std::fs::write(&path, source).unwrap();
    let bridge = real_bridge(root.path());
    block_on(bridge.spawn()).unwrap();
    let checked = block_on(bridge.check_original_program(&original, &path)).unwrap();
    assert_diagnosing_options(&checked);
    assert_eq!(
        serde_json::to_value(checked.report()).unwrap(),
        expected(case, checked.source_uri())
    );
    assert!(checked.authored_spans().is_empty());
    assert!(core::ptr::eq(checked.projection().file(), &original));
    assert_eq!(checked.source_path(), path.canonicalize().unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
    assert_eq!(std::fs::read(&config).unwrap(), config_bytes);
    block_on(bridge.shutdown()).unwrap();
}
