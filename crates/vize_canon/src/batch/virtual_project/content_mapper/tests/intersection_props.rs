//! Complete mapper modules exercise inline/shared helpers and root-slot pairs.
use std::{path::Path, process::Command};

use serde_json::{Value, json};

use crate::batch::{
    ContentMapperTransformOptions, generate_vue_content_mapper_transform_with_options,
};
use crate::virtual_ts::helpers::SHARED_PREAMBLE_STRICT_DTS;

const FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/component-intersection-props-13689"
);
const FILES: &[&str] = &[
    "Authored.vue",
    "Child.vue",
    "Empty.vue",
    "Emit.vue",
    "Fresh.vue",
    "Second.vue",
    "Generic.vue",
    "lazy.ts",
    "Valid.vue",
    "Invalid.vue",
    "controls.ts",
];

#[test]
fn intersection_props_preserve_complete_inline_and_shared_typescript_contracts() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for shared in [false, true] {
        let project = tempfile::tempdir().unwrap();
        let directory = project.path();
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            std::fs::canonicalize(workspace.join("tests/node_modules")).unwrap(),
            directory.join("node_modules"),
        )
        .unwrap();
        #[cfg(windows)]
        junction::create(
            std::fs::canonicalize(workspace.join("tests/node_modules")).unwrap(),
            directory.join("node_modules"),
        )
        .unwrap();
        let mut options =
            ContentMapperTransformOptions::default().with_shared_virtual_module(shared);
        options.checks.check_unknown_props = true;
        let mut modules = serde_json::Map::new();
        for name in FILES {
            let source =
                std::fs::read_to_string(Path::new(FIXTURES).join(format!("{name}.txt"))).unwrap();
            if name.ends_with(".vue") {
                let result = generate_vue_content_mapper_transform_with_options(
                    Path::new(name),
                    &source,
                    options,
                )
                .unwrap();
                assert!(result.diagnostics.is_empty(), "{name}: {result:?}");
                if *name == "Valid.vue" {
                    assert_eq!(result.text.matches("_instances = ").count(), 2);
                }
                let generated = directory.join(format!("{name}{}", result.extension));
                std::fs::write(&generated, result.text.as_str()).unwrap();
                modules.insert(
                    directory.join(name).to_string_lossy().into_owned(),
                    json!(generated),
                );
            } else {
                std::fs::write(directory.join(name), source).unwrap();
            }
        }
        std::fs::write(
            directory.join("__vize_helpers.d.ts"),
            format!(
                "{SHARED_PREAMBLE_STRICT_DTS}\ndeclare module 'vize/virtual-ts' {{ export type * from 'vize:virtual-ts'; }}\n"
            ),
        )
        .unwrap();
        std::fs::write(directory.join("modules.json"), json!(modules).to_string()).unwrap();
        let output = Command::new("node")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/programs/check-component-intersection-props.cjs"),
            )
            .arg(&workspace)
            .arg(directory)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stderr, b"", "{output:?}");
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["typescript"], "6.0.3");
        let diagnostics = result["diagnostics"].as_array().unwrap();
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic["file"] == "Invalid.vue.tsx"),
            "shared={shared}: {result:#}"
        );
        // Raw mapper modules retain both the `satisfies` source ownership
        // witness (TS1360) and the typed value assignment (TS2322). The
        // production batch assertion separately pins all ten authored findings.
        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic["code"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [
                2345, 1360, 2322, 1360, 2322, 2353, 2345, 1360, 2322, 2353, 1360, 2322, 1360, 2322,
                2353
            ],
            "shared={shared}: {result:#}",
        );
    }
}
