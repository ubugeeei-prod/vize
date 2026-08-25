use super::{
    collect_plugin_injection_stubs, extract_plugin_provide_keys_from_source,
    render_module_augmentation_stub, render_nuxt_composition_api_augmentation_stub,
    render_nuxt_injected_properties_stub, render_nuxt_types_augmentation_stub,
};
use vize_s0::FxHashSet;

#[test]
fn scans_src_app_plugins_for_nuxt2_injections() {
    let project_root =
        std::env::temp_dir().join(format!("vize-nuxt-src-app-plugins-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    let plugin_dir = project_root.join("src/app/plugins");
    std::fs::create_dir_all(&plugin_dir).unwrap();
    std::fs::write(
        plugin_dir.join("logger.ts"),
        r#"export default (_context, inject) => {
  if (true) {
    inject("logger", {
      info(message) {
        return message.length;
      },
    });
  }
};
"#,
    )
    .unwrap();

    let mut stubs = Vec::new();
    let mut seen_names = FxHashSet::default();
    collect_plugin_injection_stubs(&project_root, &mut stubs, &mut seen_names);

    assert!(
        stubs
            .iter()
            .any(|stub| stub.contains("$logger: __VizeNuxtInjection<'$logger'>;")),
        "expected UseContextReturn injection augmentation from src/app/plugins:\n{stubs:#?}"
    );
    assert!(
        stubs
            .iter()
            .any(|stub| stub.contains("declare const $logger")),
        "expected global injection stub from src/app/plugins:\n{stubs:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn renders_use_context_injection_augmentations() {
    let globals = render_nuxt_injected_properties_stub(&["logger".into()]);
    let types = render_nuxt_types_augmentation_stub();
    let composition =
        render_module_augmentation_stub(&render_nuxt_composition_api_augmentation_stub());

    assert!(globals.contains("$logger: __VizeNuxtInjection<'$logger'>;"));
    assert!(types.contains("interface Context extends __VizeNuxtInjectedProperties"));
    assert!(composition.starts_with("// @vize-module-augmentation\n"));
    assert!(
        composition.contains("interface UseContextReturn extends __VizeNuxtInjectedProperties")
    );
}

#[test]
fn extracts_classic_nuxt2_plugin_bindings() {
    let cases = [
        (
            "direct export control",
            r#"
export default (_context, provide) => {
  provide("direct", {})
  if (true) provide(`auth`, {})
}
"#,
            vec!["direct", "auth"],
        ),
        (
            "defineNuxtPlugin control",
            r#"
export default defineNuxtPlugin((_context, provide) => {
  provide("defined", {})
  if (true) provide(`auth`, {})
})
"#,
            vec!["defined", "auth"],
        ),
        (
            "typed const binding",
            r#"
const plugin: Plugin = (_context, provide) => provide("auth", {})
export default plugin
"#,
            vec!["auth"],
        ),
        (
            "named function binding",
            r#"
export default plugin
function plugin(_context, provide) { provide("logger", {}) }
"#,
            vec!["logger"],
        ),
        (
            "function expression binding",
            r#"
const plugin: Plugin = function (_context, provide) { provide("dayjs", {}) }
export default plugin
"#,
            vec!["dayjs"],
        ),
        (
            "wrapped const binding",
            r#"
const plugin = (((_context, provide) => provide("gtm", {})) satisfies Plugin)!
export default (plugin as Plugin)
"#,
            vec!["gtm"],
        ),
    ];

    for (name, source, expected) in cases {
        assert_eq!(
            extract_plugin_provide_keys_from_source(source),
            expected,
            "{name}"
        );
    }
}

#[test]
fn ignores_non_static_nuxt2_plugin_bindings() {
    let cases = [
        r#"
let plugin: Plugin = (_context, provide) => provide("mutable", {})
export default plugin
"#,
        r#"
import plugin from "./external"
export default (plugin satisfies Plugin)
"#,
        r#"
const implementation = (_context, provide) => provide("alias", {})
const plugin = implementation
export default plugin
"#,
        r#"
const plugin = enabled
  ? (_context, provide) => provide("enabled", {})
  : (_context, provide) => provide("disabled", {})
export default plugin
"#,
    ];

    for source in cases {
        assert!(
            extract_plugin_provide_keys_from_source(source).is_empty(),
            "unexpected static inference for:\n{source}"
        );
    }
}
