//! Historical Batch errors and genuine native pull warnings/hints stay distinct.

use super::*;
use std::os::unix::fs::PermissionsExt;

const SOURCE: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/authored-unused-symbols/App.vue.txt"
);

fn owned_backend(root: &Path) -> (PathBuf, PathBuf) {
    let pid = root.join("diagnosing.pid");
    let wrapper = root.join("tsc");
    let quote = |path: &Path| cstr!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"));
    std::fs::write(
        &wrapper,
        cstr!(
            "#!/bin/sh\nif [ \"$1\" = \"--lsp\" ]; then echo \"$$\" > {}; fi\nexec {} \"$@\"\n",
            quote(&pid),
            quote(&backend())
        ),
    )
    .unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
    (wrapper, pid)
}

#[test]
fn primitive_native_history_keeps_full_unused_errors_and_disabled_hints() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/typechecker/authored-unused-symbols");
    for (case, enabled) in [
        ("original-locals", true),
        ("inherited-locals", true),
        ("disabled-locals", false),
        ("default-locals", false),
    ] {
        let arena = Allocator::default();
        let root = tempfile::TempDir::new().unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::write(root.path().join("src/seed.ts"), "export {};\n").unwrap();
        let path = root.path().join("src/App.vue");
        std::fs::write(&path, SOURCE).unwrap();
        let config_name = if case == "original-locals" {
            String::from("original-tsconfig.json.txt")
        } else {
            cstr!("{case}.json.txt")
        };
        let configuration = std::fs::read(fixtures.join(config_name.as_str())).unwrap();
        std::fs::write(root.path().join("tsconfig.json"), &configuration).unwrap();
        if matches!(case, "inherited-locals" | "disabled-locals") {
            std::fs::copy(
                fixtures.join("base.json.txt"),
                root.path().join("base.json"),
            )
            .unwrap();
        }
        let (executable, pid) = owned_backend(root.path());
        let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(executable),
            working_dir: Some(root.path().to_path_buf()),
            ..Default::default()
        });
        let observed = lower_sfc_native(&arena, SOURCE, options());
        let admitted = observed.admitted().unwrap();
        assert!(admitted.file().file().is_complete());
        block_on(bridge.spawn()).unwrap();
        let checked = block_on(bridge.check_native_vue(admitted, &path)).unwrap();
        assert!(std::ptr::eq(
            checked.projection().original().observation(),
            &observed
        ));
        assert!(std::ptr::eq(
            checked.projection().file(),
            observed.file().unwrap().file()
        ));
        let effective = checked
            .configuration()
            .options
            .get("noUnusedLocals")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        assert_eq!(effective, enabled);
        for state in [
            checked.diagnosing_configuration().before(),
            checked.diagnosing_configuration().after(),
        ] {
            assert_eq!(
                state.project().compiler_options,
                checked.configuration().options
            );
        }
        assert_eq!(
            serde_json::to_value(checked.report()).unwrap(),
            serde_json::json!({
                "kind":"full", "items":[{
                    "range":{"start":{"line":2,"character":6},"end":{"line":2,"character":17}},
                    "severity":if enabled {2} else {4}, "code":6133,"source":"ts",
                    "message":"'unusedLocal' is declared but its value is never read."
                }]
            }),
            "{case}: whole unmasked raw report"
        );
        let start = SOURCE.find("unusedLocal").unwrap() as u32;
        assert_eq!(checked.authored_spans(), [Ok(Span::new(start, start + 11))]);
        assert_eq!(std::fs::read(&path).unwrap(), SOURCE.as_bytes());
        assert_eq!(
            std::fs::read(root.path().join("tsconfig.json")).unwrap(),
            configuration
        );
        assert!(!root.path().join("src/App.vue.ts").exists());
        {
            let pid: i32 = std::fs::read_to_string(pid)
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            // SAFETY: signal zero only observes this owned diagnosing exec child.
            assert_ne!(
                unsafe { libc::kill(pid, 0) },
                0,
                "diagnosing process survived publication"
            );
        }
        block_on(bridge.shutdown()).unwrap();
    }
}
