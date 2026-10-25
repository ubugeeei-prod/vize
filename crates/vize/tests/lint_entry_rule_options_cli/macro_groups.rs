use super::{output_details, write_file};
use crate::support::vite_config;
use serde_json::{Value, json};
use std::{fs, process::Command};

mod pinned;

const RULE: &str = "script/define-macros-order";
const ABOVE: &str = "`defineModel` should be above `defineProps`.";
const MODEL_TOP: &str = "`defineModel` should be placed at the top of `<script setup>` (after any potential import statements or type definitions).";
const PROPS_TOP: &str = "`defineProps` should be placed at the top of `<script setup>` (after any potential import statements or type definitions).";

struct Case {
    body: &'static [&'static str],
    violation: Option<(usize, &'static str)>,
}

fn cases() -> Vec<Case> {
    vec![
        Case {
            body: &[
                "const keyword = defineModel<string>('keyword', { required: true });",
                "const filter = defineModel<string>('filter', { required: true });",
                "const { label } = defineProps<{ label: string }>();",
            ],
            violation: None,
        },
        Case {
            body: &[
                "defineOptions({});",
                "defineModel();",
                "defineModel();",
                "defineProps();",
                "defineEmits();",
            ],
            violation: None,
        },
        Case {
            body: &["defineModel();", "defineProps();", "defineModel();"],
            violation: Some((2, ABOVE)),
        },
        Case {
            body: &["defineProps();", "defineModel();", "defineModel();"],
            violation: Some((1, ABOVE)),
        },
        Case {
            body: &[
                "defineModel();",
                "ordinary();",
                "defineModel();",
                "defineProps();",
            ],
            violation: Some((2, MODEL_TOP)),
        },
        Case {
            body: &[
                "defineModel();",
                "defineModel();",
                "ordinary();",
                "defineProps();",
            ],
            violation: Some((3, PROPS_TOP)),
        },
        Case {
            body: &[
                "defineModel();",
                "defineModel();",
                "defineSlots();",
                "defineProps();",
            ],
            violation: Some((3, PROPS_TOP)),
        },
    ]
}

fn check(order: Option<Value>, severity: &str, cases: &[Case], newline: &str, profile: bool) {
    let project = tempfile::tempdir().unwrap();
    let mut config = json!({"linter":{"preset":"incremental","rules":{RULE:severity}}});
    if let Some(order) = order {
        config["linter"]["ruleOptions"] = json!({RULE:{"order":order}});
    }
    write_file(
        project.path(),
        "vite.config.mjs",
        &vite_config::source(config.to_string()),
    );
    let mut expected = Vec::new();
    let mut paths = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let file = format!("src/case-{index:02}.vue");
        let source = format!(
            "<script setup lang=\"ts\">{newline}// 🦀 source coordinates{newline}{}{newline}</script>{newline}<template><div /></template>{newline}",
            case.body.join(newline)
        );
        write_file(project.path(), &file, &source);
        let mut messages = Vec::new();
        if let Some((index, message)) = case.violation {
            let statement = case.body.get(index).unwrap();
            messages.push(json!({
                "ruleId":RULE,"ruleDocsPath":"docs/content/rules/type-and-script.md",
                "severity":if severity == "error" {2} else {1},
                "message":format!("[vize:{RULE}] {message}"),
                "line":index+3,"column":1,"endLine":index+3,"endColumn":statement.len()+1,
                "help":"Declare defineProps and then defineEmits first in <script setup>, right after imports and type declarations."
            }));
        }
        expected.push(json!({"file":file,"messages":messages,"errorCount":usize::from(case.violation.is_some() && severity == "error"),"warningCount":usize::from(case.violation.is_some() && severity == "warn")}));
        paths.push(file);
    }
    let run = |profiling| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(project.path())
            .args(["lint", "--config", "vite.config.mjs", "--format", "json"])
            .args(&paths);
        if profiling {
            command.args(["--profile-json", "profile.json"]);
        }
        command.output().unwrap()
    };
    let plain = run(false);
    let exit = i32::from(severity == "error" && cases.iter().any(|case| case.violation.is_some()));
    assert_eq!(
        plain.status.code(),
        Some(exit),
        "{}",
        output_details(&plain)
    );
    let actual: Value = serde_json::from_slice(&plain.stdout).unwrap();
    assert_eq!(actual, json!(expected), "{}", output_details(&plain));
    if profile {
        let profiled = run(true);
        assert_eq!(profiled.status.code(), plain.status.code());
        assert_eq!(profiled.stdout, plain.stdout);
        let measured: Value =
            serde_json::from_slice(&fs::read(project.path().join("profile.json")).unwrap())
                .unwrap();
        let spans = measured["spans"].as_array().unwrap();
        let calls: u64 = spans
            .iter()
            .filter(|span| span["key"] == "patina.script_rule.define_macros_order")
            .map(|span| span["count"].as_u64().unwrap())
            .sum();
        assert_eq!(
            calls,
            cases.len() as u64,
            "one rule call per actual SFC script"
        );
        assert_eq!(
            measured["truncation"],
            json!({"dropped_spans":0,"dropped_counters":0})
        );
        eprintln!(
            "actual-cli-macro-rule-calls={calls}, files={}, diagnostics={}",
            cases.len(),
            cases.iter().filter(|case| case.violation.is_some()).count()
        );
    }
}

#[test]
fn repeated_macro_groups_keep_complete_cli_packets_and_one_rule_call_per_file() {
    let order = json!(["defineOptions", "defineModel", "defineProps", "defineEmits"]);
    for newline in ["\n", "\r\n"] {
        check(Some(order.clone()), "warn", &cases(), newline, true);
        check(Some(order.clone()), "error", &cases(), newline, false);
    }
}

#[test]
fn default_empty_unknown_and_project_orders_reach_the_cli_unchanged() {
    for newline in ["\n", "\r\n"] {
        check(
            None,
            "warn",
            &[
                Case {
                    body: &["defineModel();", "defineModel();", "defineProps();"],
                    violation: Some((2, PROPS_TOP)),
                },
                Case {
                    body: &[
                        "defineProps();",
                        "defineProps();",
                        "defineEmits();",
                        "defineEmits();",
                    ],
                    violation: None,
                },
            ],
            newline,
            false,
        );
        check(
            Some(json!([])),
            "warn",
            &[Case {
                body: &["defineModel();", "ordinary();", "defineProps();"],
                violation: None,
            }],
            newline,
            false,
        );
        check(
            Some(json!(["defineMissing", "defineModel", "defineProps"])),
            "warn",
            &[Case {
                body: &["defineModel();", "defineModel();", "defineProps();"],
                violation: None,
            }],
            newline,
            false,
        );
        check(
            Some(json!(["definePage", "defineModel"])),
            "warn",
            &[
                Case {
                    body: &[
                        "definePage();",
                        "definePage();",
                        "defineModel();",
                        "defineModel();",
                    ],
                    violation: None,
                },
                Case {
                    body: &["defineModel();", "definePage();", "definePage();"],
                    violation: Some((1, "`definePage` should be above `defineModel`.")),
                },
            ],
            newline,
            false,
        );
    }
}

#[test]
fn ambient_declarations_preserve_runtime_groups_and_complete_cli_packets() {
    let order = json!(["defineOptions", "defineModel", "defineProps", "defineEmits"]);
    let ambient = [
        Case {
            body: &["declare const ambient = defineModel();"],
            violation: None,
        },
        Case {
            body: &[
                "declare const ambient = defineModel();",
                "defineModel();",
                "defineModel();",
                "defineProps();",
            ],
            violation: None,
        },
        Case {
            body: &[
                "declare const ambient = withDefaults(defineModel(), {});",
                "defineModel();",
                "defineProps();",
            ],
            violation: None,
        },
        Case {
            body: &[
                "defineModel();",
                "declare const ambient = defineModel();",
                "defineModel();",
            ],
            violation: Some((2, MODEL_TOP)),
        },
        Case {
            body: &[
                "defineModel();",
                "declare const ambient: string;",
                "defineModel();",
            ],
            violation: Some((2, MODEL_TOP)),
        },
        Case {
            body: &[
                "declare const ambient = defineModel();",
                "defineProps();",
                "defineModel();",
            ],
            violation: Some((2, ABOVE)),
        },
    ];
    for newline in ["\n", "\r\n"] {
        check(Some(order.clone()), "warn", &ambient, newline, true);
        check(Some(order.clone()), "error", &ambient, newline, false);
        check(
            None,
            "warn",
            &[
                Case {
                    body: &[
                        "declare const ambient = defineProps();",
                        "defineProps();",
                        "defineEmits();",
                    ],
                    violation: None,
                },
                Case {
                    body: &[
                        "defineProps();",
                        "declare const ambient = defineProps();",
                        "defineEmits();",
                    ],
                    violation: Some((
                        2,
                        "`defineEmits` should be placed at the top of `<script setup>` (after any potential import statements or type definitions).",
                    )),
                },
            ],
            newline,
            false,
        );
    }
}
