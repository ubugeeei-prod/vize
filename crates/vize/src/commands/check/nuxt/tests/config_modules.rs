use vize_canon::virtual_ts::VirtualTsOptions;

use super::super::detect_nuxt_auto_imports;
use super::super::fallback::parse_nuxt_config_modules;
use super::unique_case_dir;

#[test]
fn ignores_comments_and_unrelated_strings() {
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
fn parses_tuple_entries_and_plain_object_export() {
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
fn parses_build_modules() {
    let modules = parse_nuxt_config_modules(
        r#"
export default defineNuxtConfig({
  modules: ['@nuxtjs/i18n'],
  buildModules: [
    '@vueuse/nuxt',
    ['@nuxtjs/color-mode', { preference: 'dark' }],
  ],
})
"#,
    );

    assert!(modules.might_include(&["@nuxtjs/i18n", "@nuxt/i18n"]));
    assert!(modules.might_include(&["@vueuse/nuxt"]));
    assert!(modules.might_include(&["@nuxtjs/color-mode"]));
    assert!(!modules.might_include(&["nuxt-og-image"]));
}

#[test]
fn detects_module_fallbacks_from_build_modules() {
    let project_root = unique_case_dir("nuxt-build-module-fallbacks");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(
        project_root.join("nuxt.config.ts"),
        r#"
export default defineNuxtConfig({
  buildModules: ['@vueuse/nuxt'],
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
            .any(|stub| stub.starts_with("declare function useClipboard<T = any")),
        "expected @vueuse/nuxt buildModule fallback stub, got: {:#?}",
        options.auto_import_stubs
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn falls_back_conservatively_for_dynamic_entries() {
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
