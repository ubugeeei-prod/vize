use serde_json::{Map, Value};

#[allow(clippy::disallowed_types)]
pub(super) fn normalize_native_removed_options(options: &mut Map<std::string::String, Value>) {
    options.remove("downlevelIteration");

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
        if module_supports_bundler_resolution(options) {
            options.insert("moduleResolution".into(), Value::String("bundler".into()));
        } else {
            options.remove("moduleResolution");
        }
    }
}

#[allow(clippy::disallowed_types)]
fn compiler_option_string<'a>(
    options: &'a Map<std::string::String, Value>,
    name: &str,
) -> Option<&'a str> {
    options.get(name).and_then(Value::as_str)
}

#[allow(clippy::disallowed_types)]
fn ascii_lowercase(value: &str) -> std::string::String {
    value.to_ascii_lowercase()
}

#[allow(clippy::disallowed_types)]
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
