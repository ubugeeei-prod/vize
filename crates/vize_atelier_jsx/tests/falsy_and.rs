//! Runtime witnesses for falsy JSX `&&` values and structural scope hygiene.
#![expect(
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "integration fixtures execute actual emitted code through Node"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx, lower_source};
use vize_l0::Allocator;
use vize_relief::{ExpressionNode, TemplateChildNode};

const FIXTURES: [(&str, &str); 4] = [
    (
        "values",
        include_str!("../../../tests/_fixtures/differential/jsx/falsy-and.jsx"),
    ),
    (
        "nested",
        include_str!("../../../tests/_fixtures/differential/jsx/falsy-and-nested.jsx"),
    ),
    (
        "boolean",
        include_str!("../../../tests/_fixtures/differential/jsx/falsy-and-boolean.jsx"),
    ),
    (
        "loop",
        include_str!("../../../tests/_fixtures/differential/jsx/falsy-and-loop.jsx"),
    ),
];

#[test]
fn actual_values_updates_and_hydration_match_jsx_child_semantics() {
    let mut fixtures = serde_json::Map::new();
    for (id, source) in FIXTURES {
        let mut outputs = serde_json::Map::new();
        for (backend, mode, ssr) in [
            ("vdom", JsxOutputMode::Vdom, false),
            ("vapor", JsxOutputMode::Vapor, false),
            ("ssr", JsxOutputMode::Vdom, true),
        ] {
            let allocator = Allocator::new();
            let output = compile_jsx(
                &allocator,
                source,
                JsxLang::Jsx,
                &JsxCompileConfig {
                    default_mode: mode,
                    ssr,
                    ..Default::default()
                },
            );
            assert!(
                !output.has_errors(),
                "{id}/{backend}: {:?}",
                output.diagnostics
            );
            assert_eq!(output.components.len(), 1, "{id}/{backend}");
            let component = &output.components[0];
            outputs.insert(
                backend.into(),
                json!({
                    "code": output.module_code(),
                    "componentCode": format!("{}\n{}", component.preamble(), component.code()),
                    "functionName": match backend { "vdom" => "App", "ssr" => "ssrRender", _ => "render" },
                    "diagnostics": output.diagnostics.iter().map(|diagnostic| json!({
                        "message": diagnostic.message.as_str(),
                        "severity": format!("{:?}", diagnostic.severity),
                        "start": diagnostic.start,
                        "end": diagnostic.end,
                    })).collect::<Vec<_>>(),
                    "map": component.map(),
                }),
            );
        }
        fixtures.insert(id.into(), json!({"source": source, "outputs": outputs}));
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = Command::new("node")
        .arg(root.join("tests/tooling/support/jsx-falsy-and-runtime.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node and installed pinned runtimes are required");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&serde_json::to_vec(&fixtures).expect("complete fixture payload"))
        .expect("send actual compiler output");
    let output = child.wait_with_output().expect("real runtime process");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).expect("runtime receipt");
    assert_eq!(receipt["failures"], 0);
    assert_eq!(receipt["nativeBytePair"], "pending");
}

#[test]
fn only_proven_booleans_use_plain_if_and_value_scopes_decline_l2_lists() {
    let cases = [
        ("true", true),
        ("!value", true),
        ("!!value", true),
        ("count > 0", true),
        ("left === right", true),
        ("'key' in value", true),
        ("value instanceof Object", true),
        ("!left && !!right", true),
        ("left ? true : false", true),
        ("flag", true),
        ("alias", true),
        ("value", false),
        ("0", false),
        ("NaN", false),
        ("Boolean(value)", false),
        ("mutable", false),
        ("object.flag", false),
        ("value as boolean", false),
        ("value && true", false),
    ];
    for (condition, boolean) in cases {
        let source = format!(
            "const flag = count > 0; const alias = flag; let mutable = true; const App = () => <div>{{{condition} && <span/>}}</div>;"
        );
        let allocator = Allocator::new();
        let output = lower_source(&allocator, allocator.as_oxc(), &source, JsxLang::Tsx);
        assert!(
            !output.has_errors(),
            "{condition}: {:?}",
            output.diagnostics
        );
        let root = &output.roots[0];
        let TemplateChildNode::Element(element) = &root.root.children[0] else {
            panic!("authored div");
        };
        if boolean {
            let TemplateChildNode::If(node) = &element.children[0] else {
                panic!("{condition} is a proven boolean");
            };
            assert_eq!(node.branches.len(), 1);
            assert!(
                root.l2.is_ok(),
                "boolean fast path keeps its native projection"
            );
        } else {
            let TemplateChildNode::For(scope) = &element.children[0] else {
                panic!("{condition} requires one lexical evaluation");
            };
            assert!(scope.parse_result.match_scope);
            assert!(scope.key_alias.is_none() && scope.object_index_alias.is_none());
            assert!(
                root.l2.is_err(),
                "synthetic lexical scope cannot become ui.for"
            );
        }
    }
}

#[test]
fn synthetic_scope_cannot_capture_an_authored_identifier() {
    // The generated binding must not capture references even inside the RHS.
    let source = "const App = () => <div>{value && <span>{_jsx_value23}</span>}</div>;";
    let allocator = Allocator::new();
    let output = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
    let TemplateChildNode::Element(element) = &output.roots[0].root.children[0] else {
        panic!("div");
    };
    let TemplateChildNode::For(scope) = &element.children[0] else {
        panic!("scope");
    };
    let Some(ExpressionNode::Simple(alias)) = &scope.value_alias else {
        panic!("alias");
    };
    assert_ne!(alias.content, "_jsx_value23");
}
