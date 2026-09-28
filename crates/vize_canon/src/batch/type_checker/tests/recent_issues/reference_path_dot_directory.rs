use super::super::{
    create_project_case_without_node_modules, resolve_test_tsgo_binary,
    snapshot_project_diagnostics,
};
use crate::batch::source_policy::SourceFilePolicy;
use crate::batch::type_checker::paths::collect_project_paths;
use crate::batch::virtual_project::VirtualProject;

#[test]
fn explicit_hidden_declaration_keeps_node_modules_reference() {
    let project_root = create_project_case_without_node_modules(
        "reference-path-dot-directory",
        &[
            (
                ".generated/env.d.ts",
                "/// <reference path=\"../node_modules/some-lib/globals.d.ts\" />\n",
            ),
            (
                "node_modules/some-lib/package.json",
                r#"{"name":"some-lib","version":"1.0.0"}"#,
            ),
            (
                "node_modules/some-lib/globals.d.ts",
                "declare const APP_VERSION: string;\n",
            ),
            (
                "src/main.ts",
                "export const version: string = APP_VERSION;\n",
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler","target":"ESNext","noEmit":true,"types":[]},"include":[".generated/env.d.ts","src/**/*"]}"#,
    )
    .unwrap();

    let mut project = VirtualProject::new(&project_root).unwrap();
    let paths = collect_project_paths(&project, SourceFilePolicy::default()).unwrap();
    assert!(paths.contains(&project_root.join(".generated/env.d.ts")));
    assert!(paths.contains(&project_root.join("src/main.ts")));
    project.set_declaration_roots(&paths);
    project.register_paths(&paths).unwrap();
    project.register_reachable_dependencies().unwrap();
    assert!(
        project
            .find_by_original(&project_root.join("node_modules/some-lib/globals.d.ts"))
            .is_some(),
        "triple-slash path targets in node_modules must be registered"
    );
    if resolve_test_tsgo_binary().is_some() {
        assert_eq!(
            snapshot_project_diagnostics(&project_root),
            Some(Vec::new())
        );
    }
    let _ = std::fs::remove_dir_all(&project_root);
}
