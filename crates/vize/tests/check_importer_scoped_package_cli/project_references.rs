use super::{install_package, resolve_test_corsa_path, run_check, workspace_root, write_file};

#[test]
fn project_references_keep_package_routes_inside_each_program() {
    let Some(corsa) = resolve_test_corsa_path() else {
        return;
    };
    let project = workspace_root()
        .join("target/vize-tests/tests")
        .join(format!(
            "importer-package-references-{}",
            std::process::id()
        ));
    let _ = std::fs::remove_dir_all(&project);
    write_file(
        &project,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./apps/alpha"},{"path":"./apps/bravo"}]}"#,
    );
    for (app, prop, ty, value) in [
        ("alpha", "alpha", "string", "\"ok\""),
        ("bravo", "bravo", "number", "1"),
    ] {
        install_package(&project, app, prop, ty);
        write_file(
            &project,
            &format!("apps/{app}/tsconfig.json"),
            r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "skipLibCheck": true,
    "composite": true,
    "customConditions": ["vize-test"]
  },
  "include": ["src/**/*.ts"]
}
"#,
        );
        write_file(
            &project,
            &format!("apps/{app}/src/entry.ts"),
            &format!(
                "import Widget from '@scope/ui'\ntype Props = InstanceType<typeof Widget>['$props']\nexport const props: Props = {{ {prop}: {value} }}\n"
            ),
        );
    }

    let output = run_check(&project, &corsa, false);
    assert!(
        output.status.success(),
        "project references leaked one package route into another:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&project);
}
