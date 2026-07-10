//! Detection of `defineNuxtPlugin` provide keys for injected helpers.

use std::path::Path;

use ignore::WalkBuilder;
use vize_carton::{FxHashSet, String, cstr};

use super::stubs::{push_stub, tracked_read_to_string};

mod extract;
#[cfg(test)]
mod tests;

pub(super) use extract::extract_plugin_provide_keys_from_source;

const MODULE_AUGMENTATION_STUB_PREFIX: &str = "// @vize-module-augmentation\n";

pub(super) fn collect_plugin_injection_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let plugin_dirs = [
        cwd.join("app/plugins"),
        cwd.join("plugins"),
        cwd.join("src/app/plugins"),
        cwd.join("src/plugins"),
    ];
    let mut plugin_keys = Vec::new();

    for dir in plugin_dirs {
        if !dir.exists() {
            continue;
        }

        let walker = WalkBuilder::new(dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if ext != "ts" && ext != "js" && ext != "mts" && ext != "cts" {
                continue;
            }

            if let Ok(source) = tracked_read_to_string(path) {
                plugin_keys.extend(extract_plugin_provide_keys_from_source(&source));
            }
        }
    }

    plugin_keys.sort();
    plugin_keys.dedup();

    if plugin_keys.is_empty() {
        return;
    }

    stubs.push(
        "type __VizeNuxtInjection<K extends PropertyKey> = import('#app').NuxtApp extends Record<K, infer T> ? T : any;"
            .into(),
    );

    stubs.push(render_nuxt_injected_properties_stub(&plugin_keys));
    if has_nuxt_types_package(cwd) {
        stubs.push(render_module_augmentation_stub(
            &render_nuxt_types_augmentation_stub(),
        ));
    } else {
        stubs.push(render_nuxt_types_augmentation_stub());
    }
    if has_nuxt_composition_api_package(cwd) {
        stubs.push(render_module_augmentation_stub(
            &render_nuxt_composition_api_augmentation_stub(),
        ));
    }

    for key in plugin_keys {
        let injected_name = if key.starts_with('$') {
            key
        } else {
            cstr!("${key}")
        };
        push_stub(
            stubs,
            seen_names,
            cstr!("declare const {injected_name}: __VizeNuxtInjection<'{injected_name}'>;"),
        );
    }
}

fn render_nuxt_injected_properties_stub(plugin_keys: &[String]) -> String {
    let mut stub = String::from("interface __VizeNuxtInjectedProperties {\n");
    for key in plugin_keys {
        let injected_name = if key.starts_with('$') {
            key.clone()
        } else {
            cstr!("${key}")
        };
        stub.push_str("  ");
        stub.push_str(injected_name.as_str());
        stub.push_str(": __VizeNuxtInjection<'");
        stub.push_str(injected_name.as_str());
        stub.push_str("'>;\n");
    }
    stub.push_str("}\n");
    stub
}

fn render_nuxt_types_augmentation_stub() -> String {
    String::from(
        "declare module \"@nuxt/types\" {\n  interface Context extends __VizeNuxtInjectedProperties {}\n  interface NuxtAppOptions extends __VizeNuxtInjectedProperties {}\n}\n",
    )
}

fn render_nuxt_composition_api_augmentation_stub() -> String {
    String::from(
        "declare module \"@nuxtjs/composition-api\" {\n  interface UseContextReturn extends __VizeNuxtInjectedProperties {}\n}\n",
    )
}

fn render_module_augmentation_stub(stub: &str) -> String {
    let mut rendered = String::from(MODULE_AUGMENTATION_STUB_PREFIX);
    rendered.push_str(stub);
    rendered
}

fn has_nuxt_types_package(cwd: &Path) -> bool {
    cwd.join("node_modules/@nuxt/types").exists()
}

fn has_nuxt_composition_api_package(cwd: &Path) -> bool {
    cwd.join("node_modules/@nuxtjs/composition-api/dist/runtime/index.d.ts")
        .is_file()
}
