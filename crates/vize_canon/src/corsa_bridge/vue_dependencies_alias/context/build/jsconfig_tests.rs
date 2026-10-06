//! JavaScript config authority precedes native editor materialization.

use crate::corsa_bridge::vue_dependencies_alias::AliasContext;
use std::path::Path;

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-jsconfig/"
);

fn fixture(root: &Path) {
    for (source, target) in [
        ("jsconfig.json.txt", "jsconfig.json"),
        ("package.json.txt", "package.json"),
        ("util.js.txt", "src/util.js"),
        ("App.vue.txt", "src/App.vue"),
        ("main.js.txt", "src/main.js"),
    ] {
        let target = root.join(target);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(Path::new(CORPUS).join(source), target).unwrap();
    }
}

#[test]
fn nearest_jsconfig_and_same_directory_tsconfig_precedence_are_preserved() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    let package = root.join("packages/app");
    fixture(&package);
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"checkJs":false}}"#,
    )
    .unwrap();
    for file in ["src/App.vue", "src/main.js"] {
        let host = package.join(file);
        let source = std::fs::read_to_string(&host).unwrap();
        let context = AliasContext::for_host(&host, &source, &Default::default());
        assert_eq!(context.project_root, package);
        assert_eq!(
            context.mirror.as_ref().unwrap().effective_tsconfig_path(),
            Some(package.join("jsconfig.json"))
        );
    }
    std::fs::copy(package.join("jsconfig.json"), package.join("tsconfig.json")).unwrap();
    let host = package.join("src/App.vue");
    let source = std::fs::read_to_string(&host).unwrap();
    let context = AliasContext::for_host(&host, &source, &Default::default());
    assert_eq!(
        context.mirror.as_ref().unwrap().effective_tsconfig_path(),
        Some(package.join("tsconfig.json"))
    );
}

#[test]
fn explicit_config_retains_authority_over_nearest_jsconfig() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    fixture(&root);
    let selected = root.join("selected.json");
    std::fs::copy(root.join("jsconfig.json"), &selected).unwrap();
    let host = root.join("src/App.vue");
    let source = std::fs::read_to_string(&host).unwrap();
    let context = super::build(
        &host,
        &source,
        &Default::default(),
        super::SourceRevision {
            requested_sources: &[],
            overlay_identity: 0,
        },
        &mut crate::PackageRouteResolver::default(),
        Default::default(),
        crate::corsa_bridge::vue_document::CorsaProjectEnvironment {
            virtual_ts_options: &Default::default(),
            package_routes: &crate::PackageRouteResolver::default(),
            project_root: Some(&root),
            tsconfig_path: Some(&selected),
            editor_session: crate::corsa_bridge::editor_session::fallback_editor_session(),
        },
    )
    .unwrap();
    assert_eq!(
        context.mirror.as_ref().unwrap().effective_tsconfig_path(),
        Some(selected)
    );
    assert!(!root.join("vize.config.json").exists());
}
