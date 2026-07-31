use serde_json::{Value, json};

use super::{option_probe_is_needed, option_probe_value};

#[allow(clippy::disallowed_types)]
fn options(value: Value) -> serde_json::Map<std::string::String, Value> {
    value.as_object().unwrap().clone()
}

#[test]
fn an_option_the_generated_config_dropped_needs_the_probe() {
    // `baseUrl` is stripped by `tsconfig_gen`, which also erases its
    // `TS5101`/`TS5102`.
    assert!(option_probe_is_needed(
        &options(json!({ "strict": true, "baseUrl": "." })),
        &options(json!({ "strict": true })),
    ));
}

#[test]
fn a_rewritten_string_option_needs_the_probe() {
    // `normalize_native_removed_options` turns `ES5` into `ES2015`, and the
    // diagnostic names the value: `TS5108: Option 'target=ES5' has been removed`.
    assert!(option_probe_is_needed(
        &options(json!({ "target": "ES5" })),
        &options(json!({ "target": "ES2015" })),
    ));
}

#[test]
fn a_re_anchored_paths_map_does_not_need_the_probe() {
    // `paths` and `typeRoots` keep their key, so every diagnostic their presence
    // triggers still fires in the main run; only their targets are re-anchored.
    assert!(!option_probe_is_needed(
        &options(json!({
            "paths": { "@/*": ["./src/*"] },
            "typeRoots": ["./types"],
        })),
        &options(json!({
            "paths": { "@/*": ["./src/*", "../../../src/*"] },
            "typeRoots": ["./types", "../../../types"],
        })),
    ));
}

#[test]
fn options_carried_through_unchanged_do_not_need_the_probe() {
    assert!(!option_probe_is_needed(
        &options(json!({ "strict": true, "target": "ES2022" })),
        &options(json!({
            "strict": true,
            "target": "ES2022",
            "allowImportingTsExtensions": true,
            "noEmit": true,
        })),
    ));
}

#[test]
fn the_probe_config_keeps_the_declared_options_and_takes_no_inputs() {
    let config = option_probe_value(options(json!({
        "baseUrl": ".",
        "strict": true,
        "types": ["node"],
    })));

    assert_eq!(
        config,
        json!({
            "compilerOptions": {
                "baseUrl": ".",
                "strict": true,
                // The only override: an unresolvable `@types` package must not
                // turn into a spurious TS2688 in a program that resolves nothing.
                "types": [],
            },
            "include": [],
            "files": [],
        })
    );
}
