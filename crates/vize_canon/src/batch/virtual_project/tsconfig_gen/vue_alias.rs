//! Expansion of tsconfig `paths` targets into virtual-mirror candidates.
//!
//! Webpack-era Vue apps import SFCs through an alias without the extension
//! (`import Head from "src/components/header/head"` for `head.vue`). Neither
//! the mirror candidate nor the real-tree fallback resolves such a specifier:
//! the mirror holds `head.vue.ts` and the real tree holds `head.vue`, and
//! TypeScript appends extensions to the specifier rather than replacing them.
//! Each alias therefore also gets a `<target>.vue.ts` mirror candidate, appended
//! after every ordinary candidate so it can only turn a previously failing
//! resolution into a success (#3300).

use std::path::Path;

use serde_json::Value;
use vize_carton::cstr;

/// Extensions that make a `paths` target name a concrete file. Such a target
/// already points at a module, so the `.vue.ts` mirror candidate would only ever
/// name a path that cannot exist.
const TARGET_SOURCE_EXTENSIONS: &[&str] = &[
    ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs", ".vue", ".json",
];

/// Expand one alias's `paths` targets into virtual-mirror candidates: the mirror
/// copy (relative to the virtual tsconfig) then the real-tree fallback for every
/// relative target, followed by the `.vue.ts` mirror candidates. `up` is the
/// relative prefix from the virtual root back to the project root.
pub(super) fn remap_path_targets(targets: &[Value], up: &str) -> Vec<Value> {
    let mut candidates = Vec::with_capacity(targets.len() * 2);
    let mut vue_candidates = Vec::new();
    for target in targets {
        let Some(target) = target.as_str() else {
            candidates.push(target.clone());
            continue;
        };
        if Path::new(target).is_absolute() {
            // Absolute targets are not mirrored, so a `.vue.ts` sibling of one
            // never exists; pass them through untouched.
            candidates.push(Value::String(target.to_owned()));
            continue;
        }
        let core = target.strip_prefix("./").unwrap_or(target);
        candidates.push(Value::String(cstr!("./{core}").into()));
        candidates.push(Value::String(cstr!("{up}{core}").into()));
        if !has_source_extension(core) {
            vue_candidates.push(Value::String(cstr!("./{core}.vue.ts").into()));
        }
    }
    candidates.extend(vue_candidates);
    candidates
}

fn has_source_extension(target: &str) -> bool {
    TARGET_SOURCE_EXTENSIONS
        .iter()
        .any(|extension| target.ends_with(extension))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::remap_path_targets;

    fn remap(targets: Value) -> Value {
        Value::Array(remap_path_targets(targets.as_array().unwrap(), "../../../"))
    }

    #[test]
    fn wildcard_alias_gains_a_trailing_vue_mirror_candidate() {
        assert_eq!(
            remap(json!(["src/*"])),
            json!(["./src/*", "../../../src/*", "./src/*.vue.ts"])
        );
    }

    #[test]
    fn extensionless_directory_alias_gains_a_vue_mirror_candidate() {
        assert_eq!(
            remap(json!(["./components/header/head"])),
            json!([
                "./components/header/head",
                "../../../components/header/head",
                "./components/header/head.vue.ts"
            ])
        );
    }

    #[test]
    fn targets_naming_a_concrete_module_keep_the_original_candidate_pair() {
        for target in [
            "./shared/index.ts",
            "shared/index.tsx",
            "shared/index.js",
            "shared/App.vue",
            "shared/data.json",
            "shared/index.mjs",
        ] {
            let remapped = remap(json!([target]));
            assert_eq!(
                remapped.as_array().unwrap().len(),
                2,
                "{target} must not gain a .vue.ts candidate"
            );
        }
    }

    #[test]
    fn vue_candidates_never_precede_an_ordinary_candidate() {
        // A `.vue.ts` mirror candidate must not shadow a later target that the
        // user declared as a fallback, so every one of them is appended last.
        assert_eq!(
            remap(json!(["primary/*", "secondary/*"])),
            json!([
                "./primary/*",
                "../../../primary/*",
                "./secondary/*",
                "../../../secondary/*",
                "./primary/*.vue.ts",
                "./secondary/*.vue.ts"
            ])
        );
    }

    #[test]
    fn absolute_and_non_string_targets_pass_through() {
        let separator = if cfg!(windows) {
            "C:\\lib\\*"
        } else {
            "/lib/*"
        };
        assert_eq!(remap(json!([separator])), json!([separator]));
        assert_eq!(remap(json!([42])), json!([42]));
    }
}
