use super::*;
use vize_canon::{NativeVueError, OriginalProgramError};

#[test]
fn actual_configuration_membership_inherited_options_and_source_guards_are_authoritative() {
    let source = "<template>{{value.missing}}</template><script setup>/** @type {number} */ let value='bad';</script>";
    let arena = Allocator::default();
    let observed = lower_sfc_native(&arena, source, options());
    let root = tempfile::TempDir::new().unwrap();
    std::fs::write(root.path().join("base.json"), serde_json::to_vec(&serde_json::json!({"compilerOptions":{"strict":true,"types":[],"allowJs":true,"checkJs":false,"module":"ESNext","target":"ESNext","moduleResolution":"Bundler"}})).unwrap()).unwrap();
    let config = serde_json::json!({"extends":"./base.json","include":["**/*"]});
    let bridge = project(root.path(), config);
    let path = root.path().join("Source.vue");
    std::fs::write(&path, source).unwrap();
    block_on(bridge.spawn()).unwrap();
    let result = block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path)).unwrap();
    assert_eq!(result.configuration().options["checkJs"], false);
    let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
        result.report()
    else {
        panic!("full report")
    };
    assert_eq!(full.full_document_diagnostic_report.items, []);
    assert_eq!(
        result.authored_spans(),
        Vec::<Result<Span, vize_l4::targets::ts::MappingError>>::new()
    );
    drop(result);
    std::fs::write(root.path().join("base.json"), serde_json::to_vec(&serde_json::json!({"compilerOptions":{"strict":true,"types":[],"allowJs":true,"checkJs":true,"module":"ESNext","target":"ESNext","moduleResolution":"Bundler"}})).unwrap()).unwrap();
    let result = block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path)).unwrap();
    assert_eq!(result.configuration().options["checkJs"], true);
    let DocumentDiagnosticReportResult::Report(DocumentDiagnosticReport::Full(full)) =
        result.report()
    else {
        panic!("full report")
    };
    assert_eq!(
        full.full_document_diagnostic_report
            .items
            .iter()
            .map(|raw| raw.code.clone())
            .collect::<Vec<_>>(),
        vec![
            Some(lsp_types::NumberOrString::Number(2322)),
            Some(lsp_types::NumberOrString::Number(2339))
        ]
    );
    assert_eq!(result.authored_spans().len(), 2);
    drop(result);
    std::fs::write(&path, "changed original source").unwrap();
    assert!(matches!(
        block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path)),
        Err(NativeVueError::Identity(
            OriginalProgramError::SourceChanged
        ))
    ));
    std::fs::write(&path, source).unwrap();
    let collision = root.path().join("Source.vue.mjs");
    std::fs::write(&collision, "real existing user file").unwrap();
    assert!(matches!(
        block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path)),
        Err(NativeVueError::ProjectionCollision)
    ));
    assert_eq!(
        std::fs::read_to_string(&collision).unwrap(),
        "real existing user file"
    );
    block_on(bridge.shutdown()).unwrap();

    for config in [
        serde_json::json!({"compilerOptions":{"allowJs":true,"checkJs":true},"files":["seed.ts"]}),
        serde_json::json!({"compilerOptions":{"allowJs":true,"checkJs":true},"include":["**/*"],"exclude":["*.vue.mjs"]}),
        serde_json::json!({"compilerOptions":{"allowJs":false,"checkJs":false},"include":["**/*"]}),
    ] {
        let root = tempfile::TempDir::new().unwrap();
        let bridge = project(root.path(), config);
        let path = root.path().join("Source.vue");
        std::fs::write(&path, source).unwrap();
        block_on(bridge.spawn()).unwrap();
        assert!(matches!(
            block_on(bridge.check_native_vue(observed.admitted().unwrap(), &path)),
            Err(NativeVueError::UnconfiguredProjection)
        ));
        assert_eq!(std::fs::read(&path).unwrap(), source.as_bytes());
        assert!(!root.path().join("Source.vue.mjs").exists());
        block_on(bridge.shutdown()).unwrap();
    }
}

#[test]
fn authored_absolute_original_source_and_original_refusals_cannot_be_bypassed() {
    let arena = Allocator::default();
    let root = tempfile::TempDir::new().unwrap();
    let bridge = configured(root.path());
    let source = "<template>{{value}}</template><script setup lang=ts>const value=1;</script>";
    let path = root.path().join("Source.vue");
    std::fs::write(&path, source).unwrap();
    let observed = lower_sfc_native(&arena, source, options());
    assert!(matches!(
        block_on(bridge.check_native_vue(observed.admitted().unwrap(), Path::new("Source.vue"))),
        Err(NativeVueError::Identity(
            OriginalProgramError::InvalidSourcePath
        ))
    ));
    let elsewhere = tempfile::TempDir::new().unwrap();
    let outside = elsewhere.path().join("Source.vue");
    std::fs::write(&outside, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_native_vue(observed.admitted().unwrap(), &outside)),
        Err(NativeVueError::Identity(
            OriginalProgramError::OutsideProject
        ))
    ));
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    std::fs::write(nested.join("tsconfig.json"), "{}").unwrap();
    let nested_source = nested.join("Source.vue");
    std::fs::write(&nested_source, source).unwrap();
    assert!(matches!(
        block_on(bridge.check_native_vue(observed.admitted().unwrap(), &nested_source)),
        Err(NativeVueError::Identity(
            OriginalProgramError::MissingConfiguredProject
        ))
    ));
    let unsupported = "<template><search>{{1}}</search></template>";
    let unsupported = lower_sfc_native(&arena, unsupported, options());
    assert!(matches!(
        block_on(bridge.check_native_vue(unsupported.admitted().unwrap(), &path)),
        Err(NativeVueError::Projection(
            vize_l4::targets::ts::vue::VueProjectionError::UnsupportedTemplate
        ))
    ));
    assert!(std::ptr::eq(
        unsupported.file().unwrap().file().artifact().source(),
        unsupported.descriptor().source()
    ));
}
