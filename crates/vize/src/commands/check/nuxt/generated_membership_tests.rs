//! Generated siblings outside the configured graph cannot inject globals.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_l0::config::VueVersion;

#[test]
fn configured_generated_stubs_follow_references_and_literal_includes() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join(".nuxt/types")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("nuxt.config.ts"), "export default {};\n").unwrap();
    std::fs::write(root.join("src/main.ts"), "export {};\n").unwrap();
    std::fs::write(
        root.join(".nuxt/nuxt.d.ts"),
        "/// <reference path=\"./imports.d.ts\" />\n/// <reference path=\"./types/referenced.d.ts\" />\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".nuxt/imports.d.ts"),
        "declare global { const useConfigured: () => string; }\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".nuxt/types/referenced.d.ts"),
        "declare global { const referencedGlobal: number; }\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".nuxt/types/unused.d.ts"),
        "declare global { const unusedGlobal: string; }\ndeclare module 'vue' { interface ComponentCustomProperties { $unused: string; } }\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".nuxt/stale-root.d.ts"),
        "declare global { const staleRootGlobal: boolean; }\nexport {};\n",
    )
    .unwrap();
    let config = root.join("tsconfig.json");
    for (include_unused, include_root) in [(false, false), (true, false), (false, true)] {
        let mut includes = vec!["src/**/*", ".nuxt/nuxt.d.ts"];
        if include_unused {
            includes.push(".nuxt/types/unused.d.ts");
        }
        if include_root {
            includes.push(".nuxt/stale-root.d.ts");
        }
        std::fs::write(
            &config,
            serde_json::to_string(&serde_json::json!({ "include": includes })).unwrap(),
        )
        .unwrap();
        let mut files = super::super::tsconfig_inputs::collect_default_check_files(
            &root,
            Some(&config),
            &mut super::super::tsconfig_inputs::TsconfigInputCache::default(),
        );
        files.extend(
            super::super::tsconfig_inputs::collect_hidden_ambient_declaration_files(
                &root,
                Some(&config),
                &mut super::super::tsconfig_inputs::TsconfigInputCache::default(),
            ),
        );
        let mut options = VirtualTsOptions::default();
        super::detect(
            &mut options,
            &root,
            Some(&config),
            Some(&files),
            false,
            VueVersion::V3,
        );
        let stubs = options.auto_import_stubs.join("\n");
        assert!(stubs.contains("declare const useConfigured:"));
        assert!(stubs.contains("declare const referencedGlobal: number;"));
        assert_eq!(
            stubs.contains("declare const unusedGlobal:"),
            include_unused
        );
        assert_eq!(
            stubs.contains("declare const staleRootGlobal:"),
            include_root
        );
        assert_eq!(
            options
                .template_globals
                .iter()
                .any(|global| global.name == "$unused"),
            include_unused
        );
        assert!(options.strict_instance_globals);
    }
}

#[test]
fn configured_generated_stubs_follow_custom_build_dir_outside_the_app() {
    let directory = tempfile::tempdir().unwrap();
    let base = directory.path().canonicalize().unwrap();
    let app = base.join("app");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::create_dir_all(base.join("generated/types")).unwrap();
    std::fs::write(
        app.join("nuxt.config.ts"),
        "export default defineNuxtConfig({ buildDir: '../generated' });\n",
    )
    .unwrap();
    std::fs::write(
        base.join("generated/nuxt.d.ts"),
        "/// <reference path=\"./types/imports.d.ts\" />\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        base.join("generated/types/imports.d.ts"),
        "declare global { const useOutsideBuildDir: () => string; }\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        base.join("generated/types/stale.d.ts"),
        "declare global { const staleOutsideBuildDir: string; }\nexport {};\n",
    )
    .unwrap();
    let config = app.join("tsconfig.json");
    std::fs::write(&config, r#"{"files":["../generated/nuxt.d.ts"]}"#).unwrap();
    let files = super::super::tsconfig_inputs::collect_default_check_files(
        &app,
        Some(&config),
        &mut super::super::tsconfig_inputs::TsconfigInputCache::default(),
    );
    let mut options = VirtualTsOptions::default();
    super::detect(
        &mut options,
        &app,
        Some(&config),
        Some(&files),
        false,
        VueVersion::V3,
    );
    let stubs = options.auto_import_stubs.join("\n");
    assert!(stubs.contains("declare const useOutsideBuildDir:"));
    assert!(!stubs.contains("declare const staleOutsideBuildDir:"));
    assert!(options.strict_instance_globals);
}
