//! Exact #7832 inputs with explicit, independent reference-path variants.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use std::path::{Path, PathBuf};

pub(crate) const CASES: &[(&str, bool, bool, bool, bool)] = &[
    ("original", false, false, false, false),
    ("leading-dot", true, false, false, false),
    ("nested", false, true, false, false),
    ("nested-leading-dot", true, true, false, false),
    ("aliased-side-effect", false, false, true, false),
    ("nested-aliased-side-effect", false, true, true, false),
    ("skip-lib-check", false, false, false, true),
];

pub(crate) const SOURCE: &str =
    include_str!("../../../../tests/fixtures/typechecker/reference-path-module/src/a.ts");

pub(crate) fn prepare(
    root: &Path,
    &(name, leading_dot, nested, alias, skip_lib_check): &(&str, bool, bool, bool, bool),
    invalid: bool,
) -> Vec<PathBuf> {
    let environment = if nested { ".nuxt/env.d.ts" } else { "env.d.ts" };
    let builder = if nested {
        ".nuxt/types/builder-env.d.ts"
    } else {
        "types/builder-env.d.ts"
    };
    let mut reference = vize_l0::String::from(include_str!(
        "../../../../tests/fixtures/typechecker/reference-path-module/env.d.ts"
    ));
    if leading_dot {
        reference = reference.replace("path=\"types/", "path=\"./types/").into();
    }
    let side_effect = if alias {
        "import \"my-client\";\n"
    } else if nested {
        "import \"../../vendor/client/index\";\n"
    } else {
        include_str!(
            "../../../../tests/fixtures/typechecker/reference-path-module/types/builder-env.d.ts"
        )
    };
    let original_config =
        include_str!("../../../../tests/fixtures/typechecker/reference-path-module/tsconfig.json");
    let mut config: serde_json::Value = serde_json::from_str(original_config).unwrap();
    config["include"][0] = environment.into();
    if alias {
        config["compilerOptions"]["paths"] =
            serde_json::json!({"my-client":["./vendor/client/index"]});
    }
    if skip_lib_check {
        config["compilerOptions"]["skipLibCheck"] = true.into();
    }
    let config = if name == "original" {
        original_config.as_bytes().to_vec()
    } else {
        serde_json::to_vec(&config).unwrap()
    };
    let source = if invalid {
        SOURCE.replace("text: string", "text: number").into()
    } else {
        vize_l0::String::from(SOURCE)
    };
    let files = [
        ("tsconfig.json", config.as_slice()),
        (environment, reference.as_bytes()),
        (builder, side_effect.as_bytes()),
        (
            "vendor/client/index.d.ts",
            include_bytes!(
                "../../../../tests/fixtures/typechecker/reference-path-module/vendor/client/index.d.ts"
            ),
        ),
        ("src/a.ts", source.as_bytes()),
    ];
    files
        .into_iter()
        .map(|(name, bytes)| {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, bytes).unwrap();
            path
        })
        .collect()
}
