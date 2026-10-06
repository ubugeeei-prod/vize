use super::*;

#[test]
fn jsconfig_collection_matches_own_defaults_and_inherited_filename_options() {
    for (name, parent_name, parent, own, expected) in [
        (
            "jsconfig.json",
            "base.json",
            r#"{"allowJs":false}"#,
            "{}",
            true,
        ),
        (
            "jsconfig.json",
            "base.json",
            r#"{"allowJs":true}"#,
            r#"{"allowJs":false}"#,
            false,
        ),
        ("tsconfig.json", "jsconfig.json", "{}", "{}", true),
        ("tsconfig.json", "base.json", "{}", "{}", false),
    ] {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("src")).unwrap();
        fs::write(
            root.path().join("src/main.ts"),
            "export const typed = true;\n",
        )
        .unwrap();
        for filename in ["a.js", "b.mjs", "c.cjs"] {
            fs::write(
                root.path().join("src").join(filename),
                "export const js = true;\n",
            )
            .unwrap();
        }
        fs::write(
            root.path().join(parent_name),
            format!(r#"{{"compilerOptions":{parent}}}"#),
        )
        .unwrap();
        let config = root.path().join(name);
        fs::write(
            &config,
            format!(
                r#"{{"extends":"./{parent_name}","compilerOptions":{own},"include":["src/**/*"]}}"#
            ),
        )
        .unwrap();
        let files = collect_default_check_files(root.path(), Some(&config));
        assert_eq!(
            relative_paths(root.path(), &files),
            if expected {
                vec!["src/a.js", "src/b.mjs", "src/c.cjs", "src/main.ts"]
            } else {
                vec!["src/main.ts"]
            },
        );
    }
}
