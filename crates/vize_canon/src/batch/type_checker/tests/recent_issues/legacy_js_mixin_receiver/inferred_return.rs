//! Preserve inferred JavaScript return types through the inherited-method bridge.

use super::super::super::{
    BatchTypeChecker, create_project_case, relative_path, resolve_test_tsgo_binary,
};
use crate::batch::TypeChecker;

const FILES: &[(&str, &str)] = &[
    (
        "src/Invalid.vue",
        include_str!(
            "../../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-return-11780/src/Invalid.vue.txt"
        ),
    ),
    (
        "src/Unicode.vue",
        include_str!(
            "../../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-return-11780/src/Unicode.vue.txt"
        ),
    ),
    (
        "src/Valid.vue",
        include_str!(
            "../../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-return-11780/src/Valid.vue.txt"
        ),
    ),
    (
        "src/mixin.js",
        include_str!(
            "../../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-return-11780/src/mixin.js.txt"
        ),
    ),
    (
        "tsconfig.json",
        include_str!(
            "../../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-return-11780/tsconfig.json.txt"
        ),
    ),
    (
        "vite.config.mjs",
        include_str!(
            "../../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-return-11780/vite.config.mjs.txt"
        ),
    ),
];
const ANY_RETURN: &str = include_str!(
    "../../../../../../../../tests/_fixtures/differential/typechecker/legacy-js-mixin-return-11780/controls/mixin-any.js.txt"
);

#[test]
fn javascript_mixin_returns_retain_inference_and_detect_authored_any_in_both_strict_modes() {
    let Some(provider) = resolve_test_tsgo_binary() else {
        return;
    };
    let original_mixin = FILES
        .iter()
        .find_map(|(file, source)| (*file == "src/mixin.js").then_some(*source))
        .unwrap();
    for strict in [false, true] {
        let root = create_project_case("legacy-js-mixin-return-11780", FILES);
        let config_path = root.join("tsconfig.json");
        let mut config: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        config["compilerOptions"]["strict"] = serde_json::Value::Bool(strict);
        std::fs::write(&config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();

        for (phase, source, string_return) in [
            ("inferred-string", original_mixin, true),
            ("authored-any", ANY_RETURN, false),
            ("restored-string", original_mixin, true),
        ] {
            std::fs::write(root.join("src/mixin.js"), source).unwrap();
            let mut checker = BatchTypeChecker::new(&root).expect("batch checker construction");
            checker.enable_legacy_vue2();
            checker.scan_project().expect("project scan");
            let result = checker.check_project().expect("native project check");
            let mut projected_vue_files: Vec<_> = checker
                .virtual_files()
                .into_iter()
                .filter(|file| {
                    file.original_path
                        .extension()
                        .is_some_and(|ext| ext == "vue")
                })
                .map(|file| relative_path(&root, &file.original_path))
                .collect();
            projected_vue_files.sort();
            assert_eq!(
                projected_vue_files,
                vec!["src/Invalid.vue", "src/Unicode.vue", "src/Valid.vue"]
            );
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
            whole.sort_by(|left, right| {
                (&left.0, left.3, left.4).cmp(&(&right.0, right.3, right.4))
            });
            if let Some(capture) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
                let capture = std::path::PathBuf::from(capture)
                    .join("legacy-js-mixin-return-11780")
                    .join(if strict { "strict" } else { "non-strict" })
                    .join(phase);
                std::fs::create_dir_all(&capture).unwrap();
                let checked_inputs: Vec<_> = FILES
                    .iter()
                    .map(|(file, _)| (*file, std::fs::read_to_string(root.join(file)).unwrap()))
                    .collect();
                let packet = serde_json::json!({
                    "provider": provider,
                    "strict": strict,
                    "phase": phase,
                    "exitCode": result.exit_code,
                    "success": result.success,
                    "projectedVueFiles": projected_vue_files,
                    "fixtureInputs": FILES,
                    "anyReturnControl": ANY_RETURN,
                    "checkedInputs": checked_inputs,
                    "mappedDiagnostics": whole,
                });
                std::fs::write(
                    capture.join("mapped-diagnostics.json"),
                    serde_json::to_vec_pretty(&packet).unwrap(),
                )
                .unwrap();
            }
            let message =
                "Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?";
            let expected = if string_return {
                vec![
                    (
                        "src/Invalid.vue".into(),
                        Some(2551),
                        message.into(),
                        7,
                        27,
                        1,
                        Some("script"),
                    ),
                    (
                        "src/Unicode.vue".into(),
                        Some(2551),
                        message.into(),
                        8,
                        27,
                        1,
                        Some("script"),
                    ),
                ]
            } else {
                vec![]
            };
            assert_eq!(whole, expected, "strict={strict}, phase={phase}");
            assert_eq!(result.exit_code, i32::from(string_return));
            assert_eq!(result.success, !string_return);
        }
        assert_eq!(
            std::fs::read_to_string(root.join("src/mixin.js")).unwrap(),
            original_mixin,
            "restoration must preserve the exact authored JavaScript"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
