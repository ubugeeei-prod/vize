use serde_json::{Map, Value};

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(super) fn normalize_native_removed_options(options: &mut Map<std::string::String, Value>) {
    options.remove("downlevelIteration");

    // TypeScript 6 flipped `noUncheckedSideEffectImports` on by default and
    // the native checker inherits that, so a side-effect import of a plain
    // asset (`import "./x.css"`) reported `TS2882` where the project's own
    // stable `tsc` reports nothing (#4964). Pin the stable default; a project
    // that declares the option keeps its setting either way.
    options
        .entry("noUncheckedSideEffectImports")
        .or_insert(Value::Bool(false));

    if matches!(
        compiler_option_string(options, "target").map(ascii_lowercase),
        Some(target) if target == "es3" || target == "es5"
    ) {
        options.insert("target".into(), Value::String("ES2015".into()));
    }

    let legacy_node_resolution = matches!(
        compiler_option_string(options, "moduleResolution").map(ascii_lowercase),
        Some(resolution) if resolution == "node" || resolution == "node10"
    );
    if legacy_node_resolution {
        // tsgo/TypeScript 7 removed the `node10` spelling. Preserve its native
        // package semantics with the supported package-map switches: this is
        // still TypeScript's resolver, with exports/imports explicitly out of
        // scope exactly as they were under Node10.
        options.insert("resolvePackageJsonExports".into(), Value::Bool(false));
        options.insert("resolvePackageJsonImports".into(), Value::Bool(false));
        if module_supports_bundler_resolution(options) {
            // Native TypeScript no longer accepts legacy `node10`, but plain
            // `bundler` would start respecting package `exports`. Keep the
            // Node-era package lookup behavior by disabling export/import-map
            // package resolution only for configs that explicitly asked for
            // legacy Node resolution.
            options.insert("moduleResolution".into(), Value::String("bundler".into()));
        } else {
            options.remove("moduleResolution");
        }
    }
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn compiler_option_string<'a>(
    options: &'a Map<std::string::String, Value>,
    name: &str,
) -> Option<&'a str> {
    options.get(name).and_then(Value::as_str)
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn ascii_lowercase(value: &str) -> std::string::String {
    value.to_ascii_lowercase()
}

#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
fn module_supports_bundler_resolution(options: &Map<std::string::String, Value>) -> bool {
    compiler_option_string(options, "module")
        .map(ascii_lowercase)
        .is_some_and(|module| {
            matches!(
                module.as_str(),
                "es6" | "es2015" | "es2020" | "es2022" | "esnext" | "preserve"
            )
        })
}
