//! Generated siblings outside the configured graph cannot inject globals.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_l0::config::VueVersion;

fn assert_complete_stubs(options: &VirtualTsOptions, mut expected: Vec<&str>) {
    expected.extend([
        "type Composer = any;",
        "type Ref<T = any> = import('vue').Ref<T>;",
        "type ComputedRef<T = any> = import('vue').ComputedRef<T>;",
        "type WritableComputedRef<T = any> = import('vue').WritableComputedRef<T>;",
        "type ShallowRef<T = any> = import('vue').ShallowRef<T>;",
        "type UnwrapRef<T> = import('vue').UnwrapRef<T>;",
        "type UnwrapNestedRefs<T> = import('vue').UnwrapNestedRefs<T>;",
        "type MaybeRef<T = any> = import('vue').MaybeRef<T>;",
        "type MaybeRefOrGetter<T = any> = import('vue').MaybeRefOrGetter<T>;",
        "type Component = import('vue').Component;",
        "declare function defineNuxtConfig<T extends Record<string, any>>(config: T): T;",
        "declare const NuxtLink: any;",
        "declare const NuxtPage: any;",
        "declare const NuxtLayout: any;",
        "declare const NuxtLoadingIndicator: any;",
        "declare const NuxtErrorBoundary: any;",
        "declare const NuxtWelcome: any;",
        "declare const NuxtIsland: any;",
        "declare const NuxtRouteAnnouncer: any;",
        "declare const ClientOnly: any;",
        "declare const DevOnly: any;",
    ]);
    let mut actual = options
        .auto_import_stubs
        .iter()
        .map(|stub| stub.as_str())
        .collect::<Vec<_>>();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

fn assert_complete_template_globals(options: &VirtualTsOptions, expected: Vec<(&str, &str, &str)>) {
    let actual = options
        .template_globals
        .iter()
        .map(|global| {
            (
                global.name.as_str(),
                global.type_annotation.as_str(),
                global.default_value.as_str(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

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
            false,
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
        let mut expected = vec![
            "declare const useConfigured: () => string;",
            "declare const referencedGlobal: number;",
        ];
        if include_unused {
            expected.push("declare const unusedGlobal: string;");
        }
        if include_root {
            expected.push("declare const staleRootGlobal: boolean;");
        }
        assert_complete_stubs(&options, expected);
        assert_complete_template_globals(
            &options,
            if include_unused {
                vec![("$unused", "any", "undefined as any")]
            } else {
                vec![]
            },
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
        false,
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
    assert_complete_stubs(
        &options,
        vec!["declare const useOutsideBuildDir: () => string;"],
    );
    assert_complete_template_globals(&options, vec![]);
    assert!(options.strict_instance_globals);
}

#[test]
fn configured_pure_manifest_retains_imports_after_ambient_root_filtering() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join(".nuxt/types")).unwrap();
    std::fs::write(root.join("nuxt.config.ts"), "export default {};\n").unwrap();
    std::fs::write(
        root.join(".nuxt/nuxt.d.ts"),
        "/// <reference path=\"./imports.d.ts\" />\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".nuxt/imports.d.ts"),
        "declare global { const useConfigured: () => string; }\nexport {};\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".nuxt/types/stale.d.ts"),
        "declare global { const staleGlobal: string; }\nexport {};\n",
    )
    .unwrap();
    let config = root.join(".nuxt/tsconfig.app.json");
    std::fs::write(&config, r#"{"include":["./nuxt.d.ts"]}"#).unwrap();
    let files = super::super::tsconfig_inputs::collect_ambient_declaration_files(
        &root,
        Some(&config),
        &mut super::super::tsconfig_inputs::TsconfigInputCache::default(),
    );
    assert_eq!(
        files,
        Vec::<std::path::PathBuf>::new(),
        "pure/import manifests are not ambient roots"
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
    assert_complete_stubs(&options, vec!["declare const useConfigured: () => string;"]);
    assert_complete_template_globals(&options, vec![]);
    assert!(options.strict_instance_globals);
}
