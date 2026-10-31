//! Effective native enum values preserve the same finite export condition sets.

use serde_json::json;

use super::condition_sets;

#[test]
fn only_known_native_export_modes_and_complete_custom_conditions_are_admitted() {
    for (numeric, named) in [(3, "Node16"), (99, "NodeNext"), (100, "Bundler")] {
        let native = json!({"moduleResolution":numeric,"customConditions":["development"]});
        let text = json!({"moduleResolution":named,"customConditions":["development"]});
        let expected = if numeric == 100 {
            vec![vec!["types", "import", "development"]]
        } else {
            vec![
                vec!["types", "node", "import", "development"],
                vec!["types", "node", "require", "development"],
            ]
        };
        assert_eq!(
            condition_sets(native.as_object().unwrap()).unwrap(),
            expected
        );
        assert_eq!(condition_sets(text.as_object().unwrap()).unwrap(), expected);
    }
    for (compiler, code) in [
        (json!({}), "unsupported-resolution"),
        (json!({"moduleResolution":2}), "unsupported-resolution"),
        (json!({"moduleResolution":777}), "unsupported-resolution"),
        (json!({"moduleResolution":-1}), "unsupported-resolution"),
        (json!({"moduleResolution":100.5}), "unsupported-resolution"),
        (json!({"moduleResolution":true}), "unsupported-resolution"),
        (
            json!({"moduleResolution":100,"resolvePackageJsonExports":false}),
            "exports-disabled",
        ),
        (
            json!({"moduleResolution":100,"customConditions":true}),
            "invalid-custom-conditions",
        ),
        (
            json!({"moduleResolution":100,"customConditions":["development",1]}),
            "invalid-custom-conditions",
        ),
        (
            json!({"moduleResolution":100,"baseUrl":"."}),
            "unsupported-base-url",
        ),
        (
            json!({"moduleResolution":100,"baseUrl":""}),
            "unsupported-base-url",
        ),
        (
            json!({"moduleResolution":100,"baseUrl":null}),
            "unsupported-base-url",
        ),
        (
            json!({"moduleResolution":100,"baseUrl":true}),
            "unsupported-base-url",
        ),
        (
            json!({"moduleResolution":100,"paths":null}),
            "unsupported-paths",
        ),
        (
            json!({"moduleResolution":100,"paths":[]}),
            "unsupported-paths",
        ),
        (
            json!({"moduleResolution":100,"paths":{"vue-router/experimental":["redirected.d.ts"]}}),
            "unsupported-paths",
        ),
        (
            json!({"moduleResolution":100,"paths":{"@/*":["src/*"]}}),
            "unsupported-paths",
        ),
        (
            json!({"moduleResolution":100,"moduleSuffixes":[".native",""]}),
            "unsupported-module-suffixes",
        ),
        (
            json!({"moduleResolution":100,"moduleSuffixes":[""]}),
            "unsupported-module-suffixes",
        ),
        (
            json!({"moduleResolution":100,"moduleSuffixes":[]}),
            "unsupported-module-suffixes",
        ),
        (
            json!({"moduleResolution":100,"moduleSuffixes":null}),
            "unsupported-module-suffixes",
        ),
        (
            json!({"moduleResolution":100,"preserveSymlinks":true}),
            "unsupported-preserve-symlinks",
        ),
        (
            json!({"moduleResolution":100,"preserveSymlinks":null}),
            "unsupported-preserve-symlinks",
        ),
        (
            json!({"moduleResolution":100,"noDtsResolution":true}),
            "unsupported-no-dts-resolution",
        ),
        (
            json!({"moduleResolution":100,"noDtsResolution":null}),
            "unsupported-no-dts-resolution",
        ),
    ] {
        assert_eq!(
            condition_sets(compiler.as_object().unwrap())
                .unwrap_err()
                .code,
            code
        );
    }
    let empty_paths =
        json!({"moduleResolution":100,"paths":{},"preserveSymlinks":false,"noDtsResolution":false});
    assert_eq!(
        condition_sets(empty_paths.as_object().unwrap()).unwrap(),
        vec![vec!["types", "import"]]
    );
}
