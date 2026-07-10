use std::path::Path;

use vize_canon::virtual_ts::VirtualTsOptions;

use super::super::detect_nuxt_auto_imports;
use super::super::generated_dir::resolve_nuxt_generated_dir;
use super::super::missing_generated_types_warning;
use super::unique_case_dir;

#[test]
fn warns_only_when_generated_nuxt_types_are_missing() {
    let generated_dir = resolve_nuxt_generated_dir(Path::new("/workspace"));
    assert!(missing_generated_types_warning(true, &generated_dir, false).is_none());
    let message = missing_generated_types_warning(false, &generated_dir, false)
        .expect("missing generated types must warn");
    assert!(message.contains("nuxi prepare"));
    assert!(message.contains("`.nuxt`"));
    assert!(message.contains("`any`"));
}

#[test]
fn detects_generated_imports_from_nuxt_config_build_dir() {
    let project_root = unique_case_dir("nuxt-config-build-dir-imports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".out/.nuxt/types")).unwrap();
    std::fs::write(
        project_root.join("nuxt.config.ts"),
        r#"export default defineNuxtConfig({ buildDir: ".out/.nuxt" })"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".out/.nuxt/types/imports.d.ts"),
        r#"declare global {
  const useCustomBuildDir: () => string
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    assert!(
        options
            .auto_import_stubs
            .iter()
            .any(|stub| stub.as_str() == "declare const useCustomBuildDir: () => string;"),
        "expected generated import from custom buildDir, got: {:#?}",
        options.auto_import_stubs
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn nuxt_config_build_dir_overrides_stale_default_imports_path() {
    let project_root = unique_case_dir("nuxt-config-build-dir-stale-imports-path");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".out/.nuxt/types")).unwrap();
    std::fs::write(
        project_root.join("nuxt.config.ts"),
        r#"export default defineNuxtConfig({ buildDir: ".out/.nuxt" })"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r##"{
  "compilerOptions": {
    "paths": {
      "#imports": [".nuxt/imports"]
    }
  }
}
"##,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".out/.nuxt/types/imports.d.ts"),
        r#"declare global {
  const useBuildDirWins: () => boolean
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    assert!(
        options
            .auto_import_stubs
            .iter()
            .any(|stub| stub.as_str() == "declare const useBuildDirWins: () => boolean;"),
        "expected generated import from nuxt.config buildDir despite stale #imports path, got: {:#?}",
        options.auto_import_stubs
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn detects_generated_imports_from_tsconfig_imports_path() {
    let project_root = unique_case_dir("nuxt-tsconfig-imports-path");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".out/.nuxt")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r##"{
  "compilerOptions": {
    "paths": {
      "#imports": [".out/.nuxt/imports"]
    }
  }
}
"##,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".out/.nuxt/imports.d.ts"),
        r#"declare global {
  const useImportsPathDir: () => number
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    assert!(
        options
            .auto_import_stubs
            .iter()
            .any(|stub| stub.as_str() == "declare const useImportsPathDir: () => number;"),
        "expected generated import from tsconfig #imports path, got: {:#?}",
        options.auto_import_stubs
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn detects_generated_imports_from_tsconfig_module_imports_path() {
    let cases = [
        ("imports.d.mts", "useImportsPathDmts"),
        ("types/imports.d.cts", "useImportsPathDcts"),
    ];

    for (target, imported_name) in cases {
        let project_root = unique_case_dir(&format!(
            "nuxt-tsconfig-module-imports-path-{imported_name}"
        ));
        let _ = std::fs::remove_dir_all(&project_root);
        let declaration_path = project_root.join(".out/.nuxt").join(target);
        std::fs::create_dir_all(declaration_path.parent().unwrap()).unwrap();
        std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
        std::fs::write(
            project_root.join("tsconfig.json"),
            format!(
                r##"{{
  "compilerOptions": {{
    "paths": {{
      "#imports": [".out/.nuxt/{target}"]
    }}
  }}
}}
"##
            ),
        )
        .unwrap();
        std::fs::write(
            declaration_path,
            format!(
                r#"declare global {{
  const {imported_name}: () => number
}}
export {{}}
"#,
            ),
        )
        .unwrap();

        let mut options = VirtualTsOptions::default();
        let _ = detect_nuxt_auto_imports(&mut options, &project_root);

        assert!(
            options.auto_import_stubs.iter().any(|stub| {
                stub.as_str() == format!("declare const {imported_name}: () => number;")
            }),
            "expected generated import from tsconfig #imports path, got: {:#?}",
            options.auto_import_stubs
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }
}

#[test]
fn warning_mentions_resolved_generated_dir() {
    let project_root = unique_case_dir("nuxt-warning-custom-build-dir");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(
        project_root.join("nuxt.config.ts"),
        r#"export default { buildDir: ".out/.nuxt" }"#,
    )
    .unwrap();

    let generated_dir = resolve_nuxt_generated_dir(&project_root);
    let message = missing_generated_types_warning(false, &generated_dir, false)
        .expect("missing generated types must warn");

    assert!(message.contains("`.out/.nuxt`"), "{message}");
    assert!(!message.contains("`.nuxt` types"), "{message}");

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn legacy_nuxt_warning_avoids_nuxt3_prepare_guidance() {
    let generated_dir = resolve_nuxt_generated_dir(Path::new("/workspace"));
    let message = missing_generated_types_warning(false, &generated_dir, true)
        .expect("missing generated types must warn");

    assert!(!message.contains("nuxi prepare"), "{message}");
    assert!(message.contains("Nuxt 2/Bridge"), "{message}");
}

#[test]
fn path_aliases_come_from_custom_generated_nuxt_tsconfig_when_present() {
    let project_root = unique_case_dir("nuxt-custom-generated-tsconfig-aliases");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".out/.nuxt")).unwrap();
    std::fs::create_dir_all(project_root.join("app")).unwrap();
    std::fs::write(
        project_root.join("nuxt.config.ts"),
        r#"export default { buildDir: ".out/.nuxt" }"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".out/.nuxt/tsconfig.json"),
        r##"{
  "compilerOptions": {
    "paths": {
      "~/*": ["../../app/*"],
      "@features/*": ["../../app/features/*"]
    }
  }
}
"##,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let aliases = detect_nuxt_auto_imports(&mut options, &project_root);

    assert!(
        aliases.iter().any(|alias| {
            alias.pattern.as_str() == "~/*"
                && alias
                    .targets
                    .iter()
                    .any(|target| target.as_str() == "app/*")
        }),
        "expected rebased ~/* alias, got: {aliases:#?}"
    );
    assert!(
        aliases.iter().any(|alias| {
            alias.pattern.as_str() == "@features/*"
                && alias
                    .targets
                    .iter()
                    .any(|target| target.as_str() == "app/features/*")
        }),
        "expected custom @features/* alias from generated tsconfig, got: {aliases:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn generated_nuxt_tsconfig_without_paths_does_not_use_guessed_aliases() {
    let project_root = unique_case_dir("nuxt-empty-generated-tsconfig-aliases");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::create_dir_all(project_root.join("app")).unwrap();
    std::fs::create_dir_all(project_root.join("shared")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/tsconfig.json"),
        r#"{ "compilerOptions": {} }"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let aliases = detect_nuxt_auto_imports(&mut options, &project_root);

    for guessed in ["~/*", "@/*", "~~/*", "@@/*", "#shared/*"] {
        assert!(
            !aliases
                .iter()
                .any(|alias| alias.pattern.as_str() == guessed),
            "generated tsconfig should suppress guessed alias {guessed:?}: {aliases:#?}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn default_generated_dir_remains_dot_nuxt() {
    let generated_dir = resolve_nuxt_generated_dir(Path::new("/workspace"));

    assert_eq!(generated_dir.display(), ".nuxt");
}
