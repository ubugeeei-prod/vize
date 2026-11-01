//! The original configured event-style fix must not cycle through formatting.
#![cfg(feature = "glyph")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "CLI observations retain complete authored inputs and outputs"
)]

use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const RULE: &str = "vue/v-on-style";
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/formatter-on-style-policy-6098/Original.vue.txt"
);
const FIXED: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/formatter-on-style-policy-6098/Fixed.vue.txt"
);
const REASON: &str = "normalizeDirectiveShorthands=true conflicts with active vue/v-on-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand event style";

fn policy() -> Value {
    json!({"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:"longform"}})
}

fn configure(root: &Path, linter: Value, normalize: Option<bool>) {
    let mut value = json!({"linter":linter});
    if let Some(normalize) = normalize {
        value["formatter"] = json!({"normalizeDirectiveShorthands":normalize});
    }
    fs::write(
        root.join("vite.config.mjs"),
        vite_config::source(value.to_string()),
    )
    .unwrap();
}

fn run(root: &Path, arguments: &[&str]) -> std::process::Output {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(arguments)
        .output()
        .unwrap();
    println!(
        "{}",
        json!({"argv":arguments,"status":output.status.code(),
        "stdout":String::from_utf8_lossy(&output.stdout),
        "stderr":String::from_utf8_lossy(&output.stderr)})
    );
    output
}

fn expect(output: &std::process::Output, status: i32) {
    assert_eq!(output.status.code(), Some(status), "{output:?}");
}

fn clean(root: &Path) {
    let output = run(
        root,
        &[
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "src/Events.vue",
        ],
    );
    expect(&output, 0);
    assert_eq!(output.stderr, b"");
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!([{"file":"src/Events.vue","messages":[],"errorCount":0,"warningCount":0}])
    );
}

#[test]
fn original_event_fix_refuses_default_and_true_then_false_stays_lint_clean_and_fixed() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let file = root.join("src/Events.vue");
    fs::write(&file, ORIGINAL).unwrap();
    configure(root, policy(), None);
    let fix = run(
        root,
        &[
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "--fix",
            "src/Events.vue",
        ],
    );
    expect(&fix, 0);
    assert_eq!(fs::read_to_string(&file).unwrap(), FIXED);
    clean(root);

    // Same-engine cycle witness, distinct from the independently checked ESLint fix.
    let shorthand = vize_glyph::format_sfc(FIXED, &vize_glyph::FormatOptions::default())
        .unwrap()
        .code;
    let linter = vize_patina::Linter::with_preset(vize_patina::LintPreset::Incremental)
        .with_additional_rules(vec![RULE.into()])
        .with_v_on_style(vize_patina::rules::vue::VOnStyleOption::Longform);
    assert_eq!(linter.lint_template(FIXED, "Events.vue").warning_count, 0);
    assert_eq!(
        linter.lint_template(&shorthand, "Events.vue").warning_count,
        2
    );
    println!(
        "{}",
        json!({"fixed":FIXED,"standaloneNormalized":shorthand.as_str()})
    );

    for normalize in [None, Some(true)] {
        configure(root, policy(), normalize);
        let config = fs::read(root.join("vite.config.mjs")).unwrap();
        for action in ["--check", "--write"] {
            let refused = run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    action,
                    "src/Events.vue",
                ],
            );
            expect(&refused, 2);
            assert_eq!(refused.stdout, b"");
            assert_eq!(
                String::from_utf8(refused.stderr).unwrap(),
                format!("Invalid formatter configuration: src/Events.vue: {REASON}\n")
            );
            assert_eq!(fs::read_to_string(&file).unwrap(), FIXED);
            assert_eq!(fs::read(root.join("vite.config.mjs")).unwrap(), config);
        }
    }
    for cli_override in [false, true] {
        configure(root, policy(), Some(cli_override));
        let mut arguments = vec![
            "fmt",
            "--config",
            "vite.config.mjs",
            "--write",
            "src/Events.vue",
        ];
        if cli_override {
            arguments.push("--normalize-directive-shorthands=false");
        }
        let output = run(root, &arguments);
        expect(&output, 0);
        let formatted = fs::read_to_string(&file).unwrap();
        let options = vize_glyph::FormatOptions {
            normalize_directive_shorthands: false,
            ..Default::default()
        };
        assert_eq!(
            formatted,
            vize_glyph::format_sfc(FIXED, &options)
                .unwrap()
                .code
                .as_str()
        );
        clean(root);
        expect(&run(root, &arguments), 0);
        assert_eq!(fs::read_to_string(&file).unwrap(), formatted);
        expect(
            &run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    "--check",
                    "src/Events.vue",
                ],
            ),
            0,
        );
        clean(root);
    }
    configure(root, policy(), Some(false));
    let before = fs::read(&file).unwrap();
    let refused = run(
        root,
        &[
            "fmt",
            "--config",
            "vite.config.mjs",
            "--write",
            "--normalize-directive-shorthands=true",
            "src/Events.vue",
        ],
    );
    expect(&refused, 2);
    assert_eq!(
        String::from_utf8(refused.stderr).unwrap(),
        format!("Invalid formatter configuration: src/Events.vue: {REASON}\n")
    );
    assert_eq!(fs::read(&file).unwrap(), before);
}

#[test]
fn selected_event_batch_is_preflighted_before_the_first_file_write() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    for name in ["A.vue", "Z.vue"] {
        fs::write(root.join("src").join(name), FIXED).unwrap();
    }
    let configuration = json!({"linter":policy(),"entries":[{"files":["src/A.vue"],"linter":{"rules":{RULE:"off"}}}]});
    fs::write(
        root.join("vite.config.mjs"),
        vite_config::source(configuration.to_string()),
    )
    .unwrap();
    let config = fs::read(root.join("vite.config.mjs")).unwrap();
    for files in [["src/A.vue", "src/Z.vue"], ["src/Z.vue", "src/A.vue"]] {
        let output = run(
            root,
            &[
                "fmt",
                "--config",
                "vite.config.mjs",
                "--write",
                files[0],
                files[1],
            ],
        );
        expect(&output, 2);
        assert_eq!(output.stdout, b"");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            format!("Invalid formatter configuration: src/Z.vue: {REASON}\n")
        );
        for name in ["A.vue", "Z.vue"] {
            assert_eq!(
                fs::read_to_string(root.join("src").join(name)).unwrap(),
                FIXED
            );
        }
        assert_eq!(fs::read(root.join("vite.config.mjs")).unwrap(), config);
    }
    fs::write(root.join("only.ts"), "const   value=1\n").unwrap();
    expect(
        &run(
            root,
            &["fmt", "--config", "vite.config.mjs", "--write", "only.ts"],
        ),
        0,
    );
    assert_eq!(
        fs::read_to_string(root.join("only.ts")).unwrap(),
        "const value = 1;\n"
    );
}

#[test]
fn event_policy_uses_the_actual_cli_rule_registration_and_base_preset() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let file = root.join("src/Events.vue");
    let expected = vize_glyph::format_sfc(FIXED, &vize_glyph::FormatOptions::default())
        .unwrap()
        .code;
    for (linter, active) in [
        (
            json!({"preset":"incremental","ruleOptions":{RULE:"longform"}}),
            true,
        ),
        (
            json!({"preset":"incremental","categories":{"style":"off"},"ruleOptions":{RULE:"longform"}}),
            true,
        ),
        (
            json!({"preset":"opinionated","categories":{"style":"off"},"ruleOptions":{RULE:"longform"}}),
            false,
        ),
        (
            json!({"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:"longform"}}),
            false,
        ),
        (
            json!({"preset":"incremental","rules":{"vue/no-v-html":"error"},"ruleOptions":{RULE:"longform"}}),
            false,
        ),
        (
            json!({"enabled":false,"preset":"opinionated","ruleOptions":{RULE:"longform"}}),
            false,
        ),
        (
            json!({"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:"shorthand"}}),
            false,
        ),
        (
            json!({"preset":"incremental","ruleOptions":{RULE:"longform","script/no-restricted-members":{"members":[{"object":"window","property":"localStorage"}]}}}),
            false,
        ),
        (
            json!({"preset":"incremental","ruleOptions":{RULE:"longform","musea/prefer-design-tokens":{"tokens":[{"path":"color.primary","value":"#3b82f6"}]}}}),
            false,
        ),
    ] {
        configure(root, linter.clone(), None);
        fs::write(&file, ORIGINAL).unwrap();
        let lint = run(
            root,
            &[
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "src/Events.vue",
            ],
        );
        assert!(matches!(lint.status.code(), Some(0 | 1)), "{lint:?}");
        let report: Value = serde_json::from_slice(&lint.stdout).unwrap();
        let messages = report[0]["messages"].as_array().unwrap();
        // Compare the actual constructor, including category-before-replacement behavior.
        let count = messages
            .iter()
            .filter(|row| {
                row["ruleId"] == RULE
                    && row["message"] == "[vize:vue/v-on-style] Prefer `v-on:` over shorthand `@`"
            })
            .count();
        assert_eq!(count, usize::from(active), "{linter}: {report}");
        fs::write(&file, FIXED).unwrap();
        let output = run(
            root,
            &[
                "fmt",
                "--config",
                "vite.config.mjs",
                "--write",
                "src/Events.vue",
            ],
        );
        expect(&output, if active { 2 } else { 0 });
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            if active { FIXED } else { expected.as_str() }
        );
    }
    for (base, entry, status) in [
        ("incremental", "opinionated", 0),
        ("opinionated", "incremental", 2),
    ] {
        let configuration = json!({"linter":{"preset":base,"rules":{"vue/no-v-html":"error"},"ruleOptions":{RULE:"longform"}},"entries":[{"files":["src/**"],"linter":{"preset":entry}}]});
        fs::write(
            root.join("vite.config.mjs"),
            vite_config::source(configuration.to_string()),
        )
        .unwrap();
        fs::write(&file, FIXED).unwrap();
        expect(
            &run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    "--write",
                    "src/Events.vue",
                ],
            ),
            status,
        );
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            if status == 2 {
                FIXED
            } else {
                expected.as_str()
            }
        );
    }
}
