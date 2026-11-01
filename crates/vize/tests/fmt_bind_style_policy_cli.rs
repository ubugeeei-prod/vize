//! Explicitly refuse a formatter policy that would undo an active lint fix.
#![cfg(feature = "glyph")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "CLI fixtures retain full original strings and reports"
)]

use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const RULE: &str = "vue/v-bind-style";
const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/formatter-bind-style-policy-6098/Original.vue.txt"
);
const CORPUS: &str = include_str!(
    "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/cases.json"
);

fn fixed_original() -> String {
    let corpus: Value = serde_json::from_str(CORPUS).unwrap();
    let expected = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["id"] == "original-longform")
        .unwrap()["reference"]["fixResult"]["output"]
        .as_str()
        .unwrap()
        .to_owned();
    let pinned = include_str!(
        "../../../tests/_fixtures/differential/linter/formatter-bind-style-policy-6098/Fixed.vue.txt"
    );
    assert_eq!(pinned, expected);
    pinned.to_owned()
}

fn lint_policy() -> Value {
    let contract: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/linter/formatter-bind-style-policy-6098/cases.json"
    ))
    .unwrap();
    contract["config"]["linter"].clone()
}

fn configure(root: &Path, linter: Value, normalize: Option<bool>) {
    let mut options = json!({"linter":linter});
    if let Some(normalize) = normalize {
        options["formatter"] = json!({"normalizeDirectiveShorthands":normalize});
    }
    fs::write(
        root.join("vite.config.mjs"),
        vite_config::source(options.to_string()),
    )
    .unwrap();
}

fn run(root: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(arguments)
        .output()
        .unwrap()
}

fn expect(output: &std::process::Output, status: i32) {
    assert_eq!(
        output.status.code(),
        Some(status),
        "stdout:{}\nstderr:{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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
            "src/VBindProp.vue",
        ],
    );
    expect(&output, 0);
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        json!([{"file":"src/VBindProp.vue","messages":[],"errorCount":0,"warningCount":0}])
    );
}

#[test]
fn original_longform_fix_conflicts_explicitly_until_normalization_is_disabled() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    assert_eq!(
        ORIGINAL,
        include_str!(
            "../../../tests/_fixtures/differential/linter/v-bind-prop-longform-6097/Original.vue.txt"
        )
    );
    fs::create_dir(root.join("src")).unwrap();
    let file = root.join("src/VBindProp.vue");
    fs::write(&file, ORIGINAL).unwrap();
    configure(root, lint_policy(), None);
    let fix = run(
        root,
        &[
            "lint",
            "--config",
            "vite.config.mjs",
            "--format",
            "json",
            "--fix",
            "src/VBindProp.vue",
        ],
    );
    expect(&fix, 0);
    let fixed = fixed_original();
    assert_eq!(fs::read_to_string(&file).unwrap(), fixed);
    clean(root);

    // This existing standalone output is the concrete cycle witness. The
    // consumer fix does not change either individual formatter option.
    let shorthand = vize_glyph::format_sfc(&fixed, &vize_glyph::FormatOptions::default())
        .unwrap()
        .code;
    assert_eq!(
        shorthand.as_str(),
        "<template>\n  <a :title.prop=\"t\"></a>\n</template>\n"
    );
    let linter = vize_patina::Linter::with_preset(vize_patina::LintPreset::Incremental)
        .with_additional_rules(vec![RULE.into()])
        .with_v_bind_style(vize_patina::rules::vue::VBindStyleOption::Longform);
    assert_eq!(
        linter.lint_template(&fixed, "VBindProp.vue").warning_count,
        0
    );
    assert_eq!(
        linter
            .lint_template(&shorthand, "VBindProp.vue")
            .warning_count,
        1
    );

    for configured in [None, Some(true)] {
        configure(root, lint_policy(), configured);
        let before = fs::read(root.join("vite.config.mjs")).unwrap();
        for action in ["--check", "--write"] {
            let refused = run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    action,
                    "src/VBindProp.vue",
                ],
            );
            expect(&refused, 2);
            assert!(refused.stdout.is_empty());
            assert_eq!(
                String::from_utf8(refused.stderr).unwrap(),
                "Invalid formatter configuration: src/VBindProp.vue: normalizeDirectiveShorthands=true conflicts with active vue/v-bind-style=longform; set formatter.normalizeDirectiveShorthands=false (CLI: --normalize-directive-shorthands=false) or use shorthand binding style\n"
            );
            assert_eq!(fs::read_to_string(&file).unwrap(), fixed);
            assert_eq!(fs::read(root.join("vite.config.mjs")).unwrap(), before);
        }
    }
    for override_cli in [false, true] {
        configure(root, lint_policy(), Some(override_cli));
        let mut arguments = vec![
            "fmt",
            "--config",
            "vite.config.mjs",
            "--write",
            "src/VBindProp.vue",
        ];
        if override_cli {
            arguments.push("--normalize-directive-shorthands=false");
        }
        let output = run(root, &arguments);
        expect(&output, 0);
        let formatted = fs::read_to_string(&file).unwrap();
        assert_eq!(
            formatted,
            "<template>\n  <a v-bind:title.prop=\"t\"></a>\n</template>\n"
        );
        clean(root);
        expect(&run(root, &arguments), 0);
        assert_eq!(fs::read_to_string(&file).unwrap(), formatted);
        clean(root);
    }
    configure(root, lint_policy(), Some(false));
    let before = fs::read(&file).unwrap();
    expect(
        &run(
            root,
            &[
                "fmt",
                "--config",
                "vite.config.mjs",
                "--write",
                "--normalize-directive-shorthands=true",
                "src/VBindProp.vue",
            ],
        ),
        2,
    );
    assert_eq!(fs::read(&file).unwrap(), before);
}

#[test]
fn entire_selected_vue_batch_is_preflighted_before_any_write() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let source = fixed_original();
    for file in ["A.vue", "Z.vue"] {
        fs::write(root.join("src").join(file), &source).unwrap();
    }
    let mut configuration = json!({"linter":lint_policy(),"entries":[{"files":["src/A.vue"],"linter":{"rules":{RULE:"off"}}}]});
    configuration["formatter"] = json!({"normalizeDirectiveShorthands":true});
    fs::write(
        root.join("vite.config.mjs"),
        vite_config::source(configuration.to_string()),
    )
    .unwrap();
    for files in [["src/A.vue", "src/Z.vue"], ["src/Z.vue", "src/A.vue"]] {
        expect(
            &run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    "--write",
                    files[0],
                    files[1],
                ],
            ),
            2,
        );
        for file in ["A.vue", "Z.vue"] {
            assert_eq!(
                fs::read_to_string(root.join("src").join(file)).unwrap(),
                source
            );
        }
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
fn entry_preset_does_not_replace_the_actual_base_preset_authority() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let file = root.join("src/VBindProp.vue");
    let fixed = fixed_original();
    for (base, entry, expected_status) in [
        ("incremental", "opinionated", 0),
        ("opinionated", "incremental", 2),
    ] {
        let configuration = json!({"linter":{"preset":base,"rules":{"vue/no-v-html":"error"},"ruleOptions":{RULE:"longform"}},"entries":[{"files":["src/**"],"linter":{"preset":entry}}]});
        fs::write(
            root.join("vite.config.mjs"),
            vite_config::source(configuration.to_string()),
        )
        .unwrap();
        fs::write(&file, &fixed).unwrap();
        expect(
            &run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    "--write",
                    "src/VBindProp.vue",
                ],
            ),
            expected_status,
        );
        if expected_status == 0 {
            let output = run(
                root,
                &[
                    "lint",
                    "--config",
                    "vite.config.mjs",
                    "--format",
                    "json",
                    "src/VBindProp.vue",
                ],
            );
            expect(&output, 0);
            assert_eq!(
                serde_json::from_slice::<Value>(&output.stdout).unwrap(),
                json!([{"file":"src/VBindProp.vue","messages":[],"errorCount":0,"warningCount":0}])
            );
        } else {
            assert_eq!(fs::read_to_string(&file).unwrap(), fixed);
            let shorthand = vize_glyph::format_sfc(&fixed, &vize_glyph::FormatOptions::default())
                .unwrap()
                .code;
            fs::write(&file, shorthand.as_bytes()).unwrap();
            let output = run(
                root,
                &[
                    "lint",
                    "--config",
                    "vite.config.mjs",
                    "--format",
                    "json",
                    "src/VBindProp.vue",
                ],
            );
            assert!(matches!(output.status.code(), Some(0 | 1)));
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            let findings = report[0]["messages"].as_array().unwrap();
            assert_eq!(
                findings
                    .iter()
                    .filter(|finding| finding["ruleId"] == RULE)
                    .count(),
                1,
                "{report}"
            );
            fs::write(&file, &fixed).unwrap();
        }
    }
}

#[test]
fn disabled_unselected_and_shorthand_policies_keep_existing_formatter_output() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let source = fixed_original();
    let expected = vize_glyph::format_sfc(&source, &vize_glyph::FormatOptions::default())
        .unwrap()
        .code;
    for linter in [
        json!({"preset":"incremental","rules":{"vue/no-v-html":"error"},"ruleOptions":{RULE:"longform"}}),
        json!({"preset":"incremental","rules":{RULE:"off"},"ruleOptions":{RULE:"longform"}}),
        json!({"enabled":false,"preset":"opinionated","ruleOptions":{RULE:"longform"}}),
        json!({"preset":"opinionated","categories":{"style":"off"},"ruleOptions":{RULE:"longform"}}),
        json!({"preset":"incremental","rules":{RULE:"error"},"ruleOptions":{RULE:"shorthand"}}),
    ] {
        fs::write(root.join("src/VBindProp.vue"), &source).unwrap();
        configure(root, linter, None);
        expect(
            &run(
                root,
                &[
                    "fmt",
                    "--config",
                    "vite.config.mjs",
                    "--write",
                    "src/VBindProp.vue",
                ],
            ),
            0,
        );
        assert_eq!(
            fs::read_to_string(root.join("src/VBindProp.vue")).unwrap(),
            expected.as_str()
        );
    }
}

#[test]
fn formatter_matches_the_existing_cli_options_only_registration_order() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    fs::create_dir(root.join("src")).unwrap();
    let file = root.join("src/VBindProp.vue");
    let source = "<template>\n  <a :title.prop=\"t\"></a>\n</template>\n";
    for categories in [json!({}), json!({"style":"off"})] {
        fs::write(&file, source).unwrap();
        configure(
            root,
            json!({"preset":"incremental","categories":categories,"ruleOptions":{RULE:"longform"}}),
            None,
        );
        let lint = run(
            root,
            &[
                "lint",
                "--config",
                "vite.config.mjs",
                "--format",
                "json",
                "src/VBindProp.vue",
            ],
        );
        expect(&lint, 0);
        assert_eq!(
            serde_json::from_slice::<Value>(&lint.stdout).unwrap(),
            json!([{
                "errorCount":0, "file":"src/VBindProp.vue", "warningCount":1,
                "messages":[{"column":6,"endColumn":21,"endLine":2,"line":2,
                    "help":"Use v-bind:attr=\"value\" instead of :attr=\"value\"",
                    "message":"[vize:vue/v-bind-style] Prefer `v-bind:` over shorthand `:`",
                    "ruleDocsPath":"docs/content/rules/vue.md","ruleId":RULE,"severity":1}]
            }])
        );
        let format = run(
            root,
            &[
                "fmt",
                "--config",
                "vite.config.mjs",
                "--write",
                "src/VBindProp.vue",
            ],
        );
        expect(&format, 2);
        assert_eq!(fs::read_to_string(&file).unwrap(), source);
    }
}
