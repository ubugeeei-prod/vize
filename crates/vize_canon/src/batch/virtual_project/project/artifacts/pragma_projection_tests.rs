//! A per-file pragma retains its complete typed projection unless JS is denied.
use super::*;

const SOURCE: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/release-js-pragma-opaque-slots/Pragma.vue.txt"
);

#[test]
fn original_pragma_projection_survives_omission_and_inherited_policy() {
    let temp = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(temp.path());
    let path = root.join("Pragma.vue");
    std::fs::write(&path, SOURCE).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"allowJs":true}}"#,
    )
    .unwrap();
    let mut reference = VirtualProject::new(&root).unwrap();
    reference.set_tsconfig_path(Some(root.join("tsconfig.json")));
    reference.register_path(&path).unwrap();
    let expected = vize_carton::cstr!("{:#?}", reference.find_by_original(&path).unwrap());
    for (base, inherited, denied) in [
        (r#"{"compilerOptions":{}}"#, false, false),
        (r#"{"compilerOptions":{}}"#, true, false),
        (r#"{"compilerOptions":{"allowJs":false}}"#, false, true),
        (r#"{"compilerOptions":{"allowJs":false}}"#, true, true),
    ] {
        std::fs::write(root.join("base.json"), base).unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            if inherited {
                r#"{"extends":"./base.json"}"#
            } else {
                base
            },
        )
        .unwrap();
        let mut project = VirtualProject::new(&root).unwrap();
        project.set_tsconfig_path(Some(root.join("tsconfig.json")));
        project.register_path(&path).unwrap();
        let file = project.find_by_original(&path).unwrap();
        if denied {
            assert_eq!(
                file.virtual_path,
                project
                    .virtual_root
                    .join("Pragma.vue.__vize_javascript__.ts")
            );
        } else {
            assert_eq!(vize_carton::cstr!("{file:#?}"), expected);
            assert!(project.unchecked_javascript_files.is_empty());
        }
        // Changing the same owner's policy must replace, not accumulate, artifacts.
        std::fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"allowJs":true}}"#,
        )
        .unwrap();
        project.refresh_compiler_configuration();
        project.register_path(&path).unwrap();
        assert_eq!(
            vize_carton::cstr!("{:#?}", project.find_by_original(&path).unwrap()),
            expected
        );
    }
}
