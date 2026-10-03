use super::*;

#[test]
fn original_jsx_path_and_configured_module_refusals_preserve_complete_owners() {
    let arena = Allocator::default();
    let source = "const view=<div/>;";
    let root = tempfile::TempDir::new().unwrap();
    configuration(root.path(), false, false);
    let bridge = bridge(root.path());
    for (profile, extension, wrong_extension) in [
        (SourceType::jsx(), "jsx", "mjs"),
        (SourceType::tsx().with_module(true), "tsx", "ts"),
    ] {
        let original = owner(&arena, source, profile);
        let path = root.path().join(cstr!("source.{extension}").as_str());
        std::fs::write(&path, source).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_jsx(&original, Path::new("source.jsx"))),
            Err(OriginalProgramError::InvalidSourcePath)
        ));
        let wrong = root.path().join(cstr!("source.{wrong_extension}").as_str());
        std::fs::write(&wrong, source).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_jsx(&original, &wrong)),
            Err(OriginalProgramError::SourceKindMismatch)
        ));
        std::fs::write(&path, "const changed=0;").unwrap();
        assert!(matches!(
            block_on(bridge.check_original_jsx(&original, &path)),
            Err(OriginalProgramError::SourceChanged)
        ));
        std::fs::write(&path, source).unwrap();
        let outside = tempfile::TempDir::new().unwrap();
        let foreign = outside.path().join(cstr!("source.{extension}").as_str());
        std::fs::write(&foreign, source).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_jsx(&original, &foreign)),
            Err(OriginalProgramError::OutsideProject)
        ));
        block_on(bridge.spawn()).unwrap();
        assert!(matches!(
            block_on(bridge.check_original_jsx(&original, &path)),
            Err(OriginalProgramError::UnsupportedModuleGoal)
        ));
        configuration(root.path(), true, true);
        assert!(matches!(
            block_on(bridge.check_original_jsx(&original, &path)),
            Err(OriginalProgramError::UnconfiguredSource)
        ));
        configuration(root.path(), false, false);
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(original.file().is_complete());
        assert_eq!(original.observation().admitted().unwrap().source(), source);
    }
    block_on(bridge.shutdown()).unwrap();
}
