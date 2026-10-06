//! #7885: real original parent/child event delivery, with whole result custody.

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_core::{
    CodegenOptions, TemplateSyntaxMode,
    steps::expression::{
        is_event_handler_reference_expression, is_function_expression,
        is_typescript_function_expression,
    },
};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};
use vize_l0::{Allocator, String, cstr};

const ROOT: &str = "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/";
const APP: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "App.vue.txt"
));
const CHILD: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Child.vue.txt"
));
const VDOM: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Vdom.vue.txt"
));
const VDOM_CHILD: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "VdomChild.vue.txt"
));
const CALLBACKS: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Callbacks.vue.txt"
));
const SPREAD: &str = include_str!(concat!(
    "../../../tests/_fixtures/differential/compiler/vapor-component-listeners/",
    "Spread.vue.txt"
));

#[test]
fn whole_function_shapes_keep_typed_callbacks_and_reject_statement_prefixes() {
    let sources = [
        "(a) => log.push(a)",
        "(a: number) => log.push(a)",
        "(a: number): void => log.push(a)",
        "async (a: number): Promise<void> => { await log(a) }",
        "function (a: number): void { log(a) }",
        "<T>(a: T) => a",
        "(a: number) => log(a) // retained comment",
        "log",
        "handlers['a;b']",
        "log(a)",
        "log(a); next()",
        "(a: number) => log(a); next()",
        "(a: number => log(a)",
    ];
    assert_eq!(
        sources.map(is_typescript_function_expression),
        [
            true, true, true, true, true, true, true, false, false, false, false, false, false
        ]
    );
    assert_eq!(
        sources.map(is_function_expression),
        [
            true, false, false, false, false, false, false, false, false, false, false, false,
            false
        ]
    );
    assert_eq!(
        sources.map(is_event_handler_reference_expression),
        [
            false, false, false, false, false, false, false, true, true, false, false, false, false
        ]
    );
}

#[test]
fn selected_component_events_keep_whole_retained_results_and_model_names() {
    for source in [
        r#"<Child @typed="(a: number) => log(a)" @update-thing="save" />"#,
        r#"<Child v-model:some-prop="value" @update-thing="save" />"#,
        r#"<Transition :css="false" @before-enter="enter"><p>{{ label }}</p></Transition>"#,
    ] {
        for prefix_identifiers in [false, true] {
            let allocator = Allocator::new();
            let result = |davinci_retained_lane| {
                let result = vize_atelier_vapor::compile_vapor(
                    &allocator,
                    source,
                    vize_atelier_vapor::VaporCompilerOptions {
                        davinci_retained_lane,
                        prefix_identifiers,
                        ..Default::default()
                    },
                );
                assert_eq!(result.error_messages, Vec::<String>::new());
                json!({ "code": result.code, "templates": result.templates,
                    "map": result.map, "errorMessages": result.error_messages })
            };
            assert_eq!(
                result(false),
                result(true),
                "{source}: prefix={prefix_identifiers}"
            );
        }
    }
}

fn compile(
    filename: &str,
    source: &str,
    vapor: bool,
    production: bool,
    output: SfcScriptOutputMode,
) -> Value {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: filename.into(),
            ..Default::default()
        },
    )
    .expect("parse whole pinned SFC");
    let mut options = SfcCompileOptions {
        vapor,
        ..Default::default()
    };
    options.parse.filename = filename.into();
    options.script.id = Some(filename.into());
    options.template.is_prod = production;
    let build = |map| {
        compile_sfc_for_adapter(
            &descriptor,
            options.clone(),
            TemplateSyntaxMode::Standard,
            Default::default(),
            CodegenOptions {
                source_map: map,
                ..Default::default()
            },
            output,
        )
        .expect("compile complete public component result")
    };
    json!({ "source": source, "filename": filename, "outputTypeScript": false,
        "plain": build(false), "mapped": build(true) })
}

fn source_custody(destination: &Path) -> Value {
    // Actions deliberately checks out depth one. Read literal parent headers
    // from the current commit object without requiring any ancestor object.
    let mut commands = Vec::new();
    let requests: [&[&str]; 2] = [&["rev-parse", "HEAD"], &["cat-file", "-p", "HEAD"]];
    for args in requests {
        let output = Command::new("git")
            .args(args)
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("capture actual compiled source identity");
        commands.push(json!({ "args": args, "exitCode": output.status.code(),
            "exitStatus": output.status.to_string(), "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "stdoutBytes": output.stdout, "stderrBytes": output.stderr }));
    }
    std::fs::write(
        destination.join("vapor-component-listeners-source.json"),
        serde_json::to_vec(&commands).expect("complete source command custody"),
    )
    .expect("retain Git exit and raw streams before identity assertions");
    assert_eq!(
        commands
            .iter()
            .map(|command| command["exitCode"].clone())
            .collect::<Vec<_>>(),
        [json!(0), json!(0)],
        "{commands:?}"
    );
    let head = commands.first().expect("head command")["stdout"]
        .as_str()
        .expect("head output")
        .trim();
    let headers: Vec<_> = commands.get(1).expect("literal object command")["stdout"]
        .as_str()
        .expect("literal commit output")
        .lines()
        .take_while(|line| !line.is_empty())
        .collect();
    let tree = headers
        .iter()
        .find_map(|line| line.strip_prefix("tree "))
        .expect("literal tree");
    let parents: Vec<_> = headers
        .iter()
        .filter_map(|line| line.strip_prefix("parent "))
        .collect();
    assert!(
        !parents.is_empty(),
        "original current source has literal parents"
    );
    for object in std::iter::once(head)
        .chain(std::iter::once(tree))
        .chain(parents.iter().copied())
    {
        assert_eq!(object.len(), 40);
        assert!(object.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    json!({ "head": head, "tree": tree, "parents": parents, "gitCommands": commands,
        "fixtureRoot": ROOT })
}

#[test]
fn original_component_listeners_mount_in_both_output_modes() {
    let profile = if std::env::var("NEXTEST_PROFILE").as_deref() == Ok("full") {
        "full"
    } else {
        "pr"
    };
    let destination = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/nextest")
        .join(profile);
    std::fs::create_dir_all(&destination).expect("whole runtime evidence directory");
    let custody = source_custody(&destination);
    let mut receipts = Vec::new();
    for production in [false, true] {
        let mut cases = Vec::new();
        for (mode, output) in [
            ("inline", SfcScriptOutputMode::InlineTemplate),
            ("separate", SfcScriptOutputMode::SeparateTemplate),
        ] {
            for (name, app, child, vapor) in [
                ("reported", APP, CHILD, true),
                ("vdom", VDOM, VDOM_CHILD, false),
                ("callbacks", CALLBACKS, CHILD, true),
                ("spread", SPREAD, CHILD, true),
            ] {
                cases.push(json!({ "name": name, "mode": mode, "vapor": vapor,
                    "app": compile("App.vue", app, vapor, production, output),
                    "child": compile("Child.vue", child, vapor, production, output) }));
            }
        }
        let input = json!({ "production": production, "cases": cases, "sourceCustody": custody });
        let label = if production {
            "production"
        } else {
            "development"
        };
        let input_path =
            destination.join(cstr!("vapor-component-listeners-input-{label}.json").as_str());
        std::fs::write(
            &input_path,
            serde_json::to_vec(&input).expect("full input JSON"),
        )
        .expect("retain complete sources and both full results before runtime assertions");
        let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/tooling/support/vapor-component-listeners.mjs");
        let mut child = Command::new("node")
            .arg(runner)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("execute actual official and source-built components");
        let write = child
            .stdin
            .take()
            .expect("child input")
            .write_all(input.to_string().as_bytes());
        let output = child
            .wait_with_output()
            .expect("retain runtime exit and streams");
        std::fs::write(
            destination.join(std::format!(
                "vapor-component-listeners-process-{label}.json"
            )),
            serde_json::to_vec(&json!({ "input": input, "exitCode": output.status.code(),
                "exitStatus": output.status.to_string(), "success": output.status.success(),
                "stdinWriteError": write.as_ref().err().map(ToString::to_string),
                "stdoutBytes": output.stdout, "stderrBytes": output.stderr }))
            .expect("complete original process custody"),
        )
        .expect("save genuine failure or success before assertion");
        assert!(write.is_ok(), "{write:?}");
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        let evidence: Value =
            serde_json::from_slice(&output.stdout).expect("whole runtime evidence");
        assert_eq!(evidence["stage"], "complete");
        assert_eq!(
            evidence["rows"]
                .as_array()
                .expect("complete row vector")
                .len(),
            8
        );
        receipts.push(evidence);
    }
    std::fs::write(
        destination.join("vapor-component-listeners-runtime.json"),
        serde_json::to_vec(&receipts).expect("both complete runtime processes"),
    )
    .expect("preserve full runtime witness alongside JUnit");
}
