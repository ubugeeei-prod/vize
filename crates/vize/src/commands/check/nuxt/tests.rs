use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::cstr;

use super::super::dts::rewrite_relative_specifier;
use super::detect_nuxt_auto_imports;
use super::fallback::{fallback_stub_strings, parse_nuxt_config_modules};
use super::missing_generated_types_warning;
use super::parsing::{parse_export_names, parse_module_specifier};
use super::plugins::extract_plugin_provide_keys_from_source;
use super::stubs::declared_name;

#[test]
fn warns_only_when_generated_nuxt_types_are_missing() {
    assert!(missing_generated_types_warning(true).is_none());
    let message =
        missing_generated_types_warning(false).expect("missing generated types must warn");
    assert!(message.contains("nuxi prepare"));
    assert!(message.contains("`any`"));
}

#[test]
fn parses_module_export_lines() {
    assert_eq!(
        parse_module_specifier("'../../app/composables/users';"),
        Some("../../app/composables/users")
    );
    assert_eq!(parse_export_names("foo as bar"), ("foo", "bar"));
    assert_eq!(parse_export_names("foo"), ("foo", "foo"));
}

#[test]
fn extracts_plugin_provide_keys_from_callback_plugin() {
    let source = r#"
export default defineNuxtPlugin(() => {
  return {
    provide: {
      scrollToTop: () => {},
      pageLifecycle: reactive({}),
    },
  }
})
"#;

    let keys = extract_plugin_provide_keys_from_source(source);
    assert_eq!(keys, vec!["scrollToTop", "pageLifecycle"]);
}

#[test]
fn extracts_plugin_provide_keys_from_setup_plugin_object() {
    let source = r#"
export default defineNuxtPlugin({
  async setup() {
    return {
      provide: {
        masto,
      },
    }
  },
})
"#;

    let keys = extract_plugin_provide_keys_from_source(source);
    assert_eq!(keys, vec!["masto"]);
}

#[test]
fn declared_name_supports_const_stubs() {
    assert_eq!(
        declared_name("declare const currentUser: any;"),
        Some("currentUser")
    );
}

#[test]
fn fallback_stub_bundle_is_valid_typescript() {
    let allocator = Allocator::default();
    let source = fallback_stub_strings().join("\n");
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);
    let ret = Parser::new(&allocator, &source, source_type).parse();

    assert!(
        ret.errors.is_empty(),
        "fallback stubs should parse as TypeScript declarations: {:#?}\n{}",
        ret.errors,
        source
    );
}

#[test]
fn relative_specifier_rewrite_matches_project_root_layout() {
    let rewritten = rewrite_relative_specifier(
        "../../app/composables/users",
        Path::new("/workspace/.nuxt/types"),
    );
    assert_eq!(rewritten.as_str(), "/workspace/app/composables/users");
}

#[test]
fn detects_nuxt_global_components_as_external_template_bindings() {
    let project_root = unique_case_dir("nuxt-components");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::create_dir_all(project_root.join("components")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/components.d.ts"),
        r#"declare module 'vue' {
  export interface GlobalComponents {
    AutoCard: typeof import('../components/AutoCard.vue')['default']
    "QuotedWidget": typeof import('../components/QuotedWidget.vue')['default']
  }
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    assert!(
        options.auto_import_stubs.iter().any(|stub| stub.contains(
            "declare const AutoCard: typeof import('./components/AutoCard.vue.ts')['default'];"
        )),
        "expected AutoCard component stub, got: {:#?}",
        options.auto_import_stubs
    );
    assert!(
        options
            .external_template_bindings
            .iter()
            .any(|name| name == "AutoCard")
    );
    assert!(
        options
            .external_template_bindings
            .iter()
            .any(|name| name == "ClientOnly")
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn detects_root_nuxt_imports_and_i18n_template_globals() {
    let project_root = unique_case_dir("nuxt-root-imports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/imports.d.ts"),
        r#"declare global {
  const useI18n: typeof import('../app/composables/i18n')['useI18n']
  const useLocalePath: typeof import('../app/composables/i18n')['useLocalePath']
  const queryCollection: typeof import('../app/composables/content')['queryCollection']
}
export {}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/i18n.d.ts"),
        r#"declare module 'vue' {
  export interface ComponentCustomProperties {
    $t: (...args: any[]) => string
  }
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    for name in ["useI18n", "useLocalePath", "queryCollection"] {
        assert!(
            options
                .auto_import_stubs
                .iter()
                .any(|stub| stub.contains(&format!("declare const {name}:"))),
            "expected {name} stub, got: {:#?}",
            options.auto_import_stubs
        );
    }
    assert!(
        options
            .template_globals
            .iter()
            .any(|global| global.name == "$t"),
        "expected $t template global, got: {:#?}",
        options.template_globals
    );
    assert!(
        options
            .template_globals
            .iter()
            .any(|global| global.name == "$te"),
        "expected i18n fallback template globals, got: {:#?}",
        options.template_globals
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn generated_imports_fall_back_to_any_for_missing_project_modules() {
    let project_root = unique_case_dir("nuxt-missing-generated-import");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::create_dir_all(project_root.join(".nuxt/composables")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/composables/useExisting.ts"),
        "export function useExisting() { return 'ok' }\n",
    )
    .unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/imports.d.ts"),
        r#"declare global {
  const useVfjsI18n: typeof import('../composables/useVfjsI18n')['useVfjsI18n']
  const useExisting: typeof import('../composables/useExisting')['useExisting']
}
export {}
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);
    let stubs = options.auto_import_stubs.join("\n");

    assert!(
        stubs.contains("declare const useVfjsI18n: any;"),
        "missing generated import should fall back to any, got:\n{stubs}"
    );
    assert!(
        stubs.contains("declare const useExisting: typeof import("),
        "existing generated import should keep its precise type, got:\n{stubs}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn detects_fallback_modules_and_aliases_without_generated_nuxt_dir() {
    let project_root = unique_case_dir("nuxt-fallback-modules");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("app/pages")).unwrap();
    std::fs::create_dir_all(project_root.join("shared")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/pages/index.vue"),
        r##"<script setup lang="ts">
import { useI18n, type Breakpoint } from "#imports";
import { VFButton } from "#components";
import { useRoute, type RoutesNamesList } from "@typed-router";
import type { NuxtError } from "#app";

void useI18n;
void VFButton;
void useRoute;
type _B = Breakpoint;
type _R = RoutesNamesList;
type _E = NuxtError;
</script>"##,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let aliases = detect_nuxt_auto_imports(&mut options, &project_root);

    assert!(aliases.iter().any(|alias| {
        alias.pattern.as_str() == "~/*"
            && alias
                .targets
                .iter()
                .any(|target| target.as_str() == "app/*")
    }));
    assert!(aliases.iter().any(|alias| {
        alias.pattern.as_str() == "~~/*"
            && alias.targets.iter().any(|target| target.as_str() == "*")
    }));
    assert!(aliases.iter().any(|alias| {
        alias.pattern.as_str() == "#shared/*"
            && alias
                .targets
                .iter()
                .any(|target| target.as_str() == "shared/*")
    }));

    let modules = options.auto_import_stubs.join("\n");
    for expected in [
        "declare module \"#imports\"",
        "export function useI18n<T = any",
        "export type Breakpoint<T = any",
        "declare module \"#components\"",
        "export const VFButton: any;",
        "declare module \"@typed-router\"",
        "export type RoutesNamesList<T = any",
        "declare module \"#app\"",
        "export type NuxtError<T = any",
    ] {
        assert!(
            modules.contains(expected),
            "expected fallback module stubs to contain {expected:?}, got:\n{modules}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn detects_source_auto_imports_without_generated_import_manifest() {
    let project_root = unique_case_dir("nuxt-source-auto-imports");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
    std::fs::create_dir_all(project_root.join("app/utils")).unwrap();
    std::fs::create_dir_all(project_root.join("shared/types")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join("app/composables/useSettings.ts"),
        r#"
export type Settings = { enabled: boolean }
export const useKeyboardShortcuts = () => true
export default function useDefaultSettings() {}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("app/utils/router.ts"),
        r#"
const localHelper = 1
export { localHelper as exportedHelper }
export const packageManagers = []
export function packageRoute() {}
"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("shared/types/social.ts"),
        "export type NPMXProfile = { displayName: string }",
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    for expected in [
        "declare const exportedHelper: any;",
        "declare const packageManagers: any;",
        "declare function packageRoute<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): any;",
        "declare function useDefaultSettings<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): ({ value: T } & Record<string, any>);",
        "declare function useKeyboardShortcuts<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): ({ value: T } & Record<string, any>);",
    ] {
        assert!(
            options
                .auto_import_stubs
                .iter()
                .any(|stub| stub == expected),
            "expected source auto-import stub {expected:?}, got: {:#?}",
            options.auto_import_stubs
        );
    }
    assert!(
        !options
            .auto_import_stubs
            .iter()
            .any(|stub| stub == "declare const Settings: any;"),
        "type-only exports should not become auto-import values: {:#?}",
        options.auto_import_stubs
    );
    assert!(
        options
            .auto_import_stubs
            .iter()
            .any(|stub| stub == "type NPMXProfile = any;"),
        "expected source type auto-import stub, got: {:#?}",
        options.auto_import_stubs
    );

    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::write(
        project_root.join(".nuxt/imports.d.ts"),
        r#"
declare global {
  const generatedOnly: any
}
export {}
"#,
    )
    .unwrap();

    let mut generated_options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut generated_options, &project_root);
    assert!(
        generated_options
            .auto_import_stubs
            .iter()
            .any(|stub| stub == "declare const generatedOnly: any;"),
        "expected generated import stub, got: {:#?}",
        generated_options.auto_import_stubs
    );
    assert!(
        !generated_options
            .auto_import_stubs
            .iter()
            .any(|stub| stub.contains(" packageRoute<")
                || stub.starts_with("declare const packageRoute:")),
        "source fallback should defer to generated import manifests: {:#?}",
        generated_options.auto_import_stubs
    );
    assert!(
        !generated_options
            .auto_import_stubs
            .iter()
            .any(|stub| stub == "type NPMXProfile = any;"),
        "source type fallback should defer to generated import manifests: {:#?}",
        generated_options.auto_import_stubs
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn skips_fallback_any_stubs_when_generated_import_manifest_exists() {
    let project_root = unique_case_dir("nuxt-no-fallback-with-generated");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    std::fs::write(
        project_root.join(".nuxt/types/imports.d.ts"),
        r#"declare global {
  const useRouter: typeof import('vue-router')['useRouter']
  const navigateTo: typeof import('../../node_modules/nuxt/dist/app/composables/router')['navigateTo']
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
            .any(|stub| stub.contains("declare const useRouter: typeof import(")),
        "expected generated useRouter stub, got: {:#?}",
        options.auto_import_stubs
    );
    let any_ladder = [
        "declare function useRouter(): any;",
        "declare function useRoute(name?: string): any;",
        "declare function navigateTo(to: string | any, options?: any): any;",
        "declare function useNuxtApp(): any;",
        "declare function watch(source: any, cb: (...args: any[]) => any, options?: any): any;",
    ];
    for fallback in any_ladder {
        assert!(
            !options
                .auto_import_stubs
                .iter()
                .any(|stub| stub == fallback),
            "fallback any-stub {fallback:?} should not be injected when .nuxt generated types exist: {:#?}",
            options.auto_import_stubs
        );
    }
    assert!(
        options
            .auto_import_stubs
            .iter()
            .any(|stub| stub == "type Ref<T = any> = import('vue').Ref<T>;"),
        "auto-imported type aliases should still be injected: {:#?}",
        options.auto_import_stubs
    );
    assert!(
        options
            .auto_import_stubs
            .iter()
            .any(|stub| stub == "declare const NuxtLink: any;"),
        "built-in component stubs should still be injected: {:#?}",
        options.auto_import_stubs
    );

    let fallback_root = unique_case_dir("nuxt-fallback-without-generated");
    let _ = std::fs::remove_dir_all(&fallback_root);
    std::fs::create_dir_all(&fallback_root).unwrap();
    std::fs::write(fallback_root.join("nuxt.config.ts"), "export default {}").unwrap();

    let mut fallback_options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut fallback_options, &fallback_root);
    for fallback in any_ladder {
        assert!(
            fallback_options
                .auto_import_stubs
                .iter()
                .any(|stub| stub == fallback),
            "fallback any-stub {fallback:?} should be injected without .nuxt generated types: {:#?}",
            fallback_options.auto_import_stubs
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
    let _ = std::fs::remove_dir_all(&fallback_root);
}

#[test]
fn detects_module_fallbacks_from_nuxt_config() {
    let project_root = unique_case_dir("nuxt-module-fallbacks");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("app/pages")).unwrap();
    std::fs::write(
        project_root.join("nuxt.config.ts"),
        r#"
export default defineNuxtConfig({
  modules: ['@nuxtjs/i18n', '@nuxt/content', '@vueuse/nuxt', '@nuxtjs/color-mode', 'nuxt-og-image', 'motion-v/nuxt'],
})
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    for expected in [
        "declare function useI18n():",
        "declare function useLocalePath<T = any",
        "declare function useLocaleHead<T = any",
        "declare function queryCollection<T = any",
        "declare function useClipboard<T = any",
        "declare function useScrollLock<T = any",
        "declare function useColorMode():",
        "declare function defineOgImageComponent<T = any",
        "declare module \"motion-v\"",
    ] {
        assert!(
            options
                .auto_import_stubs
                .iter()
                .any(|stub| stub.starts_with(expected)),
            "expected module fallback stub {expected:?}, got: {:#?}",
            options.auto_import_stubs
        );
    }
    assert!(
        options
            .template_globals
            .iter()
            .any(|global| global.name == "$t"),
        "expected i18n template globals, got: {:#?}",
        options.template_globals
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn nuxt_config_modules_ignores_comments_and_unrelated_strings() {
    let modules = parse_nuxt_config_modules(
        r#"
export default defineNuxtConfig({
  // modules: ['@nuxtjs/i18n'],
  /* '@vueuse/nuxt' would be nice */
  modules: [
    // '@nuxtjs/color-mode',
    '@nuxtjs/i18n',
  ],
  runtimeConfig: { public: { hint: "enable nuxt-og-image later" } },
})
"#,
    );

    assert!(modules.might_include(&["@nuxtjs/i18n", "@nuxt/i18n"]));
    assert!(!modules.might_include(&["@vueuse/nuxt"]));
    assert!(!modules.might_include(&["@nuxtjs/color-mode"]));
    assert!(!modules.might_include(&["nuxt-og-image"]));
}

#[test]
fn nuxt_config_modules_parses_tuple_entries_and_plain_object_export() {
    let modules = parse_nuxt_config_modules(
        r#"
export default {
  modules: [
    ['@nuxtjs/i18n', { locales: ['en'] }],
    'nuxt-og-image',
  ],
}
"#,
    );

    assert!(modules.might_include(&["@nuxtjs/i18n", "@nuxt/i18n"]));
    assert!(modules.might_include(&["nuxt-og-image"]));
    assert!(!modules.might_include(&["@vueuse/nuxt"]));

    let empty = parse_nuxt_config_modules("export default defineNuxtConfig({})");
    assert!(!empty.might_include(&["@nuxtjs/i18n", "@nuxt/i18n"]));
}

#[test]
fn nuxt_config_modules_falls_back_conservatively_for_dynamic_entries() {
    // A spread may contribute any module name, so unmatched candidates stay
    // conservatively "maybe present" while static entries still resolve.
    let spread = parse_nuxt_config_modules(
        r#"
const extras = ['@vueuse/nuxt']
export default defineNuxtConfig({
  modules: ['@nuxtjs/color-mode', ...extras],
})
"#,
    );
    assert!(spread.might_include(&["@nuxtjs/color-mode"]));
    assert!(spread.might_include(&["@vueuse/nuxt"]));
    assert!(spread.might_include(&["@nuxtjs/i18n", "@nuxt/i18n"]));

    // Computed entries and opaque default exports are equally unresolved.
    let computed =
        parse_nuxt_config_modules("export default { modules: [resolveModule('nuxt-og-image')] }");
    assert!(computed.might_include(&["nuxt-og-image"]));
    assert!(computed.might_include(&["@vueuse/nuxt"]));

    let opaque = parse_nuxt_config_modules("export default buildConfig()");
    assert!(opaque.might_include(&["nuxt-og-image"]));
}

#[test]
fn skips_module_fallbacks_for_commented_out_modules() {
    let project_root = unique_case_dir("nuxt-commented-modules");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(
        project_root.join("nuxt.config.ts"),
        r#"
export default defineNuxtConfig({
  // modules: ['@nuxtjs/i18n', '@vueuse/nuxt'],
  modules: [
    // '@nuxtjs/color-mode',
    'nuxt-og-image',
  ],
})
"#,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut options, &project_root);

    assert!(
        options
            .auto_import_stubs
            .iter()
            .any(|stub| stub.starts_with("declare function defineOgImageComponent<T = any")),
        "expected nuxt-og-image fallback stub, got: {:#?}",
        options.auto_import_stubs
    );
    for absent in [
        "declare function useI18n():",
        "declare function useColorMode():",
        "declare function useClipboard<T = any",
    ] {
        assert!(
            !options
                .auto_import_stubs
                .iter()
                .any(|stub| stub.starts_with(absent)),
            "commented-out module must not inject {absent:?}, got: {:#?}",
            options.auto_import_stubs
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn path_aliases_come_from_generated_nuxt_tsconfig_when_present() {
    let project_root = unique_case_dir("nuxt-generated-tsconfig-aliases");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
    std::fs::create_dir_all(project_root.join("app")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
    // `.nuxt/tsconfig.json` is JSON-with-comments and carries the project's REAL
    // `compilerOptions.paths`. Targets are relative to `.nuxt/`. The custom
    // `@features/*` alias would never be produced by the hardcoded guesses.
    std::fs::write(
        project_root.join(".nuxt/tsconfig.json"),
        r##"{
  // generated by nuxi prepare
  "compilerOptions": {
    "paths": {
      "~/*": ["../app/*"],
      "@features/*": ["../app/features/*"],
      "#shared/*": ["../shared/*"],
    },
  },
}
"##,
    )
    .unwrap();

    let mut options = VirtualTsOptions::default();
    let aliases = detect_nuxt_auto_imports(&mut options, &project_root);

    // Aliases come verbatim from the generated config, rebased to project root.
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
    assert!(
        aliases.iter().any(|alias| {
            alias.pattern.as_str() == "#shared/*"
                && alias
                    .targets
                    .iter()
                    .any(|target| target.as_str() == "shared/*")
        }),
        "expected #shared/* alias from generated tsconfig, got: {aliases:#?}"
    );
    // Hardcoded guesses absent from the generated config must NOT be synthesized.
    assert!(
        !aliases.iter().any(|alias| alias.pattern.as_str() == "@/*"),
        "guessed @/* alias must not be injected when generated tsconfig is present: {aliases:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn path_aliases_fall_back_to_guesses_without_generated_nuxt_tsconfig() {
    let project_root = unique_case_dir("nuxt-no-generated-tsconfig-aliases");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("app")).unwrap();
    std::fs::create_dir_all(project_root.join("shared")).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();

    let mut options = VirtualTsOptions::default();
    let aliases = detect_nuxt_auto_imports(&mut options, &project_root);

    for (pattern, target) in [
        ("~/*", "app/*"),
        ("@/*", "app/*"),
        ("#shared/*", "shared/*"),
    ] {
        assert!(
            aliases.iter().any(|alias| {
                alias.pattern.as_str() == pattern
                    && alias.targets.iter().any(|t| t.as_str() == target)
            }),
            "expected hardcoded fallback alias {pattern:?} -> {target:?}, got: {aliases:#?}"
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    workspace_root
        .join("target")
        .join("vize-tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}
