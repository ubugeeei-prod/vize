//! Plugin lint rules from `vue.plugins` (#13074): a v2 `lint.rule` and an
//! adapted v1 rule plugin run in one sidecar crossing per lint run, with
//! host-owned spans, configured severities, fixes and a cost receipt.
#![expect(clippy::disallowed_types, reason = "complete CLI process packets")]

use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

const BUTTON: &str = "<template>\n  <button :color=\"'red'\">Go</button>\n</template>\n";

fn repo(path: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

/// The P4-16 fixture source the v1 packet freeze was recorded from.
fn todo_list() -> Vec<u8> {
    let output = Command::new("node")
        .args([
            "--input-type=module",
            "-e",
            "const { TODO_LIST } = await import(process.argv[1]); process.stdout.write(TODO_LIST);",
            &repo("tests/tooling/fixtures/davinci-plugin-sdk/sources.mjs"),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    output.stdout
}

fn project(rules: &str, options: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let config = vize_l0::cstr!(
        "import {{ writeFileSync }} from \"node:fs\";\n\
         import {{ pathToFileURL }} from \"node:url\";\n\
         const load = (key) => import(pathToFileURL(process.env[key]).href);\n\
         const {{ defineVizePlugin }} = await load(\"VIZE_TEST_PLUGIN_SDK\");\n\
         const {{ default: team }} = await load(\"VIZE_TEST_TEAM_CONVENTIONS\");\n\
         const design = defineVizePlugin({{\n\
           apiVersion: 2, stability: \"experimental\", name: \"@acme/design\", version: \"1.0.0\",\n\
           setup(api, options) {{\n\
             api.lint.rule({{\n\
               id: \"no-raw-color\",\n\
               meta: {{ messages: {{ raw: \"Use {{token}} instead of {{value}}.\" }}, defaultLevel: \"warn\", fixable: \"safe\" }},\n\
               visit: [\"ui.bind\"],\n\
               run(context) {{\n\
                 for (const node of context.nodes) {{\n\
                   if (node.name === \"color\") context.report({{ node, messageId: \"raw\", data: {{ token: options.token, value: node.value }}, fix: \"color\" }});\n\
                 }}\n\
               }},\n\
             }});\n\
           }},\n\
         }});\n\
         const recorded = {{ ...team, run(batch) {{ writeFileSync(process.env.VIZE_TEST_BATCH_LOG, batch, {{ flag: \"a\" }}); return team.run(batch); }} }};\n\
         export default {{ vue: {{ plugins: [design({options}), recorded], linter: {{ preset: \"essential\", rules: {rules} }} }} }};\n"
    );
    fs::write(dir.path().join("vite.config.mjs"), config.as_str()).unwrap();
    fs::write(dir.path().join("TodoList.vue"), todo_list()).unwrap();
    fs::write(dir.path().join("Button.vue"), BUTTON).unwrap();
    dir
}

fn vize(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("VIZE_TEST_PLUGIN_SDK", repo("npm/plugin-sdk/v2.js"))
        .env(
            "VIZE_TEST_TEAM_CONVENTIONS",
            repo("tests/tooling/fixtures/davinci-plugin-sdk/team-conventions.mjs"),
        )
        .env("VIZE_TEST_BATCH_LOG", root.join("batches.log"))
        .env("VIZE_CONFIG_CACHE", "0")
        .args(args)
        .output()
        .unwrap()
}

fn plugin_messages(output: &std::process::Output) -> Value {
    let results: Value = serde_json::from_slice(&output.stdout).unwrap();
    Value::Array(
        results
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|file| {
                file["messages"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|message| {
                        let rule = message["ruleId"].as_str().unwrap_or_default();
                        rule.starts_with("@acme/") || rule.starts_with("team-conventions/")
                    })
                    .map(|message| {
                        json!([
                            file["file"],
                            message["ruleId"],
                            message["severity"],
                            message["message"],
                            message["line"],
                            message["column"],
                            message["endLine"],
                            message["endColumn"],
                        ])
                    })
                    .collect::<Vec<_>>()
            })
            .collect(),
    )
}

fn receipt(output: &std::process::Output) -> Value {
    let stderr = String::from_utf8_lossy(&output.stderr);
    stderr
        .lines()
        .find_map(|line| line.strip_prefix("vize_plugin_receipt: "))
        .map_or(Value::Null, |json| serde_json::from_str(json).unwrap())
}

#[test]
fn v2_and_adapted_v1_rules_report_with_configured_severities_and_fixes() {
    let dir = project(
        "{ \"@acme/design/no-raw-color\": \"error\" }",
        "{ token: \"$brand\" }",
    );
    let output = vize(
        dir.path(),
        &["lint", "Button.vue", "TodoList.vue", "--format", "json"],
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        plugin_messages(&output),
        json!([
            [
                "Button.vue",
                "@acme/design/no-raw-color",
                2,
                "[vize:@acme/design/no-raw-color] Use $brand instead of 'red'.",
                2,
                11,
                2,
                25
            ],
            [
                "TodoList.vue",
                "team-conventions/no-index-key",
                1,
                "[vize:team-conventions/no-index-key] Don't key a v-for by its index `i`; use a stable id such as `cell.id`.",
                4,
                42,
                4,
                50
            ],
            [
                "TodoList.vue",
                "team-conventions/no-index-key",
                1,
                "[vize:team-conventions/no-index-key] Don't key a v-for by its index `index`; use a stable id such as `todo.id`.",
                8,
                40,
                8,
                52
            ],
        ])
    );
}

#[test]
fn plugin_rules_link_the_plugin_rule_page_instead_of_a_builtin_one() {
    let dir = project("{}", "{ token: \"$brand\" }");
    let output = vize(dir.path(), &["lint", "Button.vue", "--format", "json"]);
    let results: Value = serde_json::from_slice(&output.stdout).unwrap();
    let pages: Vec<&Value> = results[0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|message| message["ruleId"] == "@acme/design/no-raw-color")
        .map(|message| &message["ruleDocsPath"])
        .collect();
    assert_eq!(
        pages,
        [&json!(
            "docs/release/integrated-plugin-incubation.md#plugin-lint-rules"
        )]
    );
}

/// The adapted v1 plugin receives the exact batch bytes the native JS host
/// sent in the v1 packet freeze.
#[test]
fn adapted_v1_plugins_receive_the_frozen_v1_batch_bytes() {
    let dir = project("{}", "{ token: \"$brand\" }");
    let output = vize(dir.path(), &["lint", "TodoList.vue", "--format", "json"]);
    // Built-in preset errors keep exit status 1; a plugin failure would be 2.
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let frozen: Value = serde_json::from_slice(
        &fs::read(repo(
            "tests/tooling/fixtures/davinci-plugin-sdk/api-v1-packets.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let case = frozen
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "lint/team-conventions")
        .unwrap();
    assert_eq!(
        fs::read_to_string(dir.path().join("batches.log")).unwrap(),
        case["packets"][0][0].as_str().unwrap()
    );
}

#[test]
fn one_sidecar_crossing_carries_every_job_and_the_receipt_attributes_cost() {
    let dir = project("{}", "{ token: \"$brand\" }");
    let output = vize(
        dir.path(),
        &[
            "lint",
            "Button.vue",
            "TodoList.vue",
            "--format",
            "json",
            "--profile",
        ],
    );
    let receipt = receipt(&output);
    assert_eq!(
        receipt["crossings"],
        1,
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(receipt["documents"], 2);
    assert_eq!(receipt["skipped"], json!([]));
    let jobs: Vec<Value> = receipt["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|job| {
            json!([
                job["plugin"],
                job["key"],
                job["batches"],
                job["nodes"],
                job["reports"]
            ])
        })
        .collect();
    assert_eq!(
        jobs,
        [
            json!(["@acme/design", "@acme/design/no-raw-color", 2, 6, 1]),
            json!(["team-conventions", "team-conventions", 2, 11, 2]),
        ]
    );
    assert_eq!(receipt["requestBytes"].as_u64().unwrap() > 0, true);
    assert_eq!(receipt["responseBytes"].as_u64().unwrap() > 0, true);
}

#[test]
fn disabled_rules_add_no_work_and_no_output() {
    let disabled = project(
        "{ \"@acme/design/no-raw-color\": \"off\", \"team-conventions/no-index-key\": \"off\" }",
        "{ token: \"$brand\" }",
    );
    let output = vize(
        disabled.path(),
        &[
            "lint",
            "Button.vue",
            "TodoList.vue",
            "--format",
            "json",
            "--profile",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(receipt(&output), Value::Null);
    assert_eq!(plugin_messages(&output), json!([]));
    assert_eq!(disabled.path().join("batches.log").exists(), false);
}

#[test]
fn a_plan_that_changes_between_evaluations_fails_closed() {
    let dir = project("{}", "{ token: String(process.hrtime.bigint()) }");
    let output = vize(dir.path(), &["lint", "Button.vue"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let refusal = stderr
        .lines()
        .find_map(|line| line.split_once("[plugin/").map(|(_, rest)| rest))
        .and_then(|rest| rest.split_once(']'))
        .map(|(code, _)| code);
    assert_eq!(refusal, Some("plan-stale"), "{stderr}");
}

#[test]
fn plugin_rules_are_configured_at_the_root_only() {
    let dir = project("{}", "{ token: \"$brand\" }");
    let config = fs::read_to_string(dir.path().join("vite.config.mjs")).unwrap().replace(
        "linter: { preset: \"essential\", rules: {} }",
        "linter: { preset: \"essential\", rules: {} }, entries: [{ files: [\"*.vue\"], linter: { rules: { \"@acme/design/no-raw-color\": \"off\" } } }]",
    );
    fs::write(dir.path().join("vite.config.mjs"), config).unwrap();
    let output = vize(dir.path(), &["lint", "Button.vue"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.lines().last(),
        Some(
            "\u{1b}[31mError:\u{1b}[0m Plugin lint rules are configured in the root vue.linter.rules, not in entries[]: @acme/design/no-raw-color"
        )
    );
}
