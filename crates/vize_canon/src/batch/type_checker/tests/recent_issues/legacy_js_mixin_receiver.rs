//! JavaScript mixin methods must not open the entire legacy receiver (#11780).

use super::super::{
    create_project_case, relative_path, resolve_test_tsgo_binary, BatchTypeChecker,
};
use crate::batch::TypeChecker;

const FILES: &[(&str, &str)] = &[
    (
        "src/Invalid.vue",
        include_str!("../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-receiver-11780/src/Invalid.vue.txt"),
    ),
    (
        "src/Unicode.vue",
        include_str!("../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-receiver-11780/src/Unicode.vue.txt"),
    ),
    (
        "src/Valid.vue",
        include_str!("../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-receiver-11780/src/Valid.vue.txt"),
    ),
    (
        "src/mixin.js",
        include_str!("../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-receiver-11780/src/mixin.js.txt"),
    ),
    (
        "tsconfig.json",
        include_str!("../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-receiver-11780/tsconfig.json.txt"),
    ),
    (
        "vite.config.mjs",
        include_str!("../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-receiver-11780/vite.config.mjs.txt"),
    ),
];

#[test]
fn javascript_mixin_methods_keep_legacy_receivers_closed_in_both_strict_modes() {
    let Some(provider) = resolve_test_tsgo_binary() else {
        return;
    };
    for strict in [false, true] {
        let root = create_project_case("legacy-js-mixin-receiver-11780", FILES);
        let config_path = root.join("tsconfig.json");
        let mut config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        config["compilerOptions"]["strict"] = serde_json::Value::Bool(strict);
        std::fs::write(&config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        let mut checker = BatchTypeChecker::new(&root).expect("batch checker construction");
        checker.enable_legacy_vue2();
        checker.scan_project().expect("project scan");
        let result = checker.check_project().expect("native project check");
        let mut whole: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    relative_path(&root, &diagnostic.file),
                    diagnostic.code,
                    diagnostic.message.clone(),
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.severity,
                    diagnostic.block_type.map(|block| block.block_name()),
                )
            })
            .collect();
        whole.sort_by(|left, right| (&left.0, left.3, left.4).cmp(&(&right.0, right.3, right.4)));
        if let Some(capture) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
            let capture = std::path::PathBuf::from(capture)
                .join("legacy-js-mixin-receiver-11780")
                .join(if strict { "strict" } else { "non-strict" });
            std::fs::create_dir_all(&capture).unwrap();
            let packet = serde_json::json!({
                "provider": provider,
                "strict": strict,
                "inputs": FILES,
                "tsconfig": config,
                "mappedDiagnostics": whole,
            });
            std::fs::write(
                capture.join("mapped-diagnostics.json"),
                serde_json::to_vec_pretty(&packet).unwrap(),
            )
            .unwrap();
        }
        let _ = std::fs::remove_dir_all(&root);
        let missing = "Property 'MISSING_EDGE_MEMBER_DO_NOT_EXIST' does not exist on type '$This'.";
        let wrong = "Argument of type 'number' is not assignable to parameter of type 'string'.";
        assert_eq!(
            whole,
            vec![
                (
                    "src/Invalid.vue".into(),
                    Some(2339),
                    missing.into(),
                    7,
                    12,
                    1,
                    Some("script")
                ),
                (
                    "src/Invalid.vue".into(),
                    Some(2345),
                    wrong.into(),
                    8,
                    18,
                    1,
                    Some("script")
                ),
                (
                    "src/Unicode.vue".into(),
                    Some(2339),
                    missing.into(),
                    8,
                    12,
                    1,
                    Some("script")
                ),
                (
                    "src/Unicode.vue".into(),
                    Some(2345),
                    wrong.into(),
                    9,
                    18,
                    1,
                    Some("script")
                ),
            ],
            "strict={strict}; valid SFC and authored JavaScript must remain diagnostic-free"
        );
    }
}
