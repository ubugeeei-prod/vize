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
            // An absolute target escaped the project root during rebase. Its
            // reachable first-party files mirror into the external escape
            // subtree (#3887), so that copy — generated `.vue.ts` companions
            // and rewritten barrels — resolves ahead of the real tree, which
            // stays as the fallback for anything not mirrored.
            candidates.push(Value::String(external_candidate(target).into()));
            candidates.push(Value::String(target.to_owned()));
            if !has_source_extension(target) {
                vue_candidates.push(Value::String(
                    cstr!("{}.vue.ts", external_candidate(target)).into(),
                ));
            }
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

/// The external-mirror candidate for an escaped absolute target, spelled
/// relative to the virtual tsconfig: the absolute path replayed as
/// subdirectories under the escape subtree, matching
/// `external_mirror::external_mirror_path`.
fn external_candidate(target: &str) -> vize_carton::String {
    let mut replayed = vize_carton::String::from("./__vize_external__");
    for component in Path::new(target).components() {
        use std::path::Component;
        match component {
            Component::Normal(part) => {
                replayed.push('/');
                replayed.push_str(&part.to_string_lossy());
            }
            Component::Prefix(prefix) => {
                replayed.push('/');
                replayed.push_str(&prefix.as_os_str().to_string_lossy().replace(':', "%3A"));
            }
            _ => {}
        }
    }
    replayed
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
    fn absolute_targets_gain_the_external_mirror_candidate_first() {
        // An absolute target escaped the project root during rebase; its
        // first-party files mirror into the external subtree (#3887), which
        // must win over the real tree so rewritten barrels and generated
        // `.vue.ts` companions resolve, while the real path stays the
        // fallback for anything not mirrored.
        if cfg!(windows) {
            return;
        }
        assert_eq!(
            remap(json!(["/ws/pkg/src/index.ts"])),
            json!([
                "./__vize_external__/ws/pkg/src/index.ts",
                "/ws/pkg/src/index.ts"
            ])
        );
        assert_eq!(
            remap(json!(["/ws/pkg/src/*"])),
            json!([
                "./__vize_external__/ws/pkg/src/*",
                "/ws/pkg/src/*",
                "./__vize_external__/ws/pkg/src/*.vue.ts"
            ])
        );
    }

    #[test]
    fn non_string_targets_pass_through() {
        assert_eq!(remap(json!([42])), json!([42]));
    }
}
