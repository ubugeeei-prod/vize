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
use vize_atelier_jsx::{
    JsxCompileConfig, JsxLang, JsxOutputMode, SsrCompileOptions, VaporCompileOptions, compile_jsx,
    compile_to_ssr, compile_to_vapor, lower_source,
};
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

// The fixed packet owns standalone renderer producers, not today's module host.
fn compile_fixtures(renderer_producers: bool) -> Value {
    let mut fixtures = serde_json::Map::new();
    for (id, source) in FIXTURES {
        let mut outputs = serde_json::Map::new();
        for (backend, mode, ssr) in [
            ("vdom", JsxOutputMode::Vdom, false),
            ("vapor", JsxOutputMode::Vapor, false),
            ("ssr", JsxOutputMode::Vdom, true),
        ] {
            let allocator = Allocator::new();
            if renderer_producers && backend != "vdom" {
                let (code, diagnostics) = if ssr {
                    let output = compile_to_ssr(
                        &allocator,
                        source,
                        JsxLang::Jsx,
                        SsrCompileOptions::default(),
                    );
                    assert!(
                        !output.has_errors(),
                        "{id}/{backend}: {:?}",
                        output.diagnostics
                    );
                    assert_eq!(output.components.len(), 1);
                    (
                        output
                            .components
                            .into_iter()
                            .next()
                            .expect("one producer")
                            .code,
                        output.diagnostics,
                    )
                } else {
                    let output = compile_to_vapor(
                        &allocator,
                        source,
                        JsxLang::Jsx,
                        VaporCompileOptions::default(),
                    );
                    assert!(
                        !output.has_errors(),
                        "{id}/{backend}: {:?}",
                        output.diagnostics
                    );
                    assert_eq!(output.components.len(), 1);
                    (
                        output
                            .components
                            .into_iter()
                            .next()
                            .expect("one producer")
                            .code,
                        output.diagnostics,
                    )
                };
                assert!(diagnostics.is_empty(), "{id}/{backend}: {diagnostics:?}");
                outputs.insert(
                    backend.into(),
                    json!({
                        "code": code, "componentCode": format!("\n{code}"),
                        "functionName": if ssr { "ssrRender" } else { "render" },
                        "diagnostics": [], "map": null,
                    }),
                );
                continue;
            }
            let output = compile_jsx(
                &allocator,
                source,
                JsxLang::Jsx,
                &JsxCompileConfig {
                    // Same authored source, with the supported SSR Vapor object role.
                    default_mode: if ssr { JsxOutputMode::Vapor } else { mode },
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
                    "functionName": "App",
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
    Value::Object(fixtures)
}

// A value-varying witness for the existing children/ternary corpus row (#6890).
// Only VDOM is qualified here; the original && witnesses retain their three backends.
fn compile_value_sensitive_ternary() -> Value {
    let source = r#"const App = () => <div>{read("condition") ? <a>{read("truthy")}</a> : <b>{read("falsy")}</b>}</div>;"#;
    let allocator = Allocator::new();
    let output = compile_jsx(
        &allocator,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: JsxOutputMode::Vdom,
            ssr: false,
            ..Default::default()
        },
    );
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    assert_eq!(output.components.len(), 1);
    let component = output.components.first().expect("one authored component");
    json!({
        "source": source,
        "corpusId": "children/ternary",
        "outputs": { "vdom": {
            "code": output.module_code(),
            "componentCode": format!("{}\n{}", component.preamble(), component.code()),
            "functionName": "App",
            "diagnostics": [],
            "map": component.map(),
        }},
        "clientVapor": "not qualified by this VDOM-only witness",
    })
}

#[test]
fn renderer_producers_match_complete_frozen_legacy_bytes_and_metadata() {
    let golden: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/jsx/falsy-and.fixed-legacy.json"
    ))
    .expect("authenticated source-built compiler fixture");
    let actual = compile_fixtures(true);
    // Capture the explicit producer contract; the historical module packet
    // remains immutable and is never regenerated from current module output.
    if let Some(path) = std::env::var_os("VIZE_FALSY_AND_CAPTURE") {
        std::fs::write(path, actual.to_string()).expect("capture writes");
    }
    let expected = &golden["fixtures"];
    assert_eq!(
        actual.as_object().expect("complete object").len(),
        expected.as_object().expect("complete object").len()
    );
    for (id, source) in FIXTURES {
        assert_eq!(
            expected[id]["source"]
                .as_str()
                .expect("complete text facet")
                .as_bytes(),
            source.as_bytes()
        );
        for backend in ["vdom", "vapor", "ssr"] {
            let output = &actual[id]["outputs"][backend];
            let fixed = &expected[id]["outputs"][backend];
            // Public text facets retain every byte, including final newlines.
            for facet in ["code", "componentCode"] {
                assert_eq!(
                    output[facet]
                        .as_str()
                        .expect("complete text facet")
                        .as_bytes(),
                    fixed[facet]
                        .as_str()
                        .expect("complete text facet")
                        .as_bytes(),
                    "{id}/{backend}/{facet}"
                );
            }
            // Complete structured facets are a separate comparison.
            for facet in ["diagnostics", "map", "functionName"] {
                assert_eq!(output[facet], fixed[facet], "{id}/{backend}/{facet}");
            }
            assert_eq!(
                output
                    .as_object()
                    .expect("complete object")
                    .keys()
                    .collect::<Vec<_>>(),
                fixed
                    .as_object()
                    .expect("complete object")
                    .keys()
                    .collect::<Vec<_>>()
            );
        }
    }
    assert_eq!(golden["native"]["handled"], 0);
    assert_eq!(golden["native"]["equivalent"], 0);
    assert_eq!(golden["native"]["pair"], "pending");
}

#[test]
fn original_functional_vdom_ssr_sources_refuse_the_complete_module() {
    let cases = FIXTURES.iter().map(|(id, source)| {
        let allocator = Allocator::new();
        let output = compile_jsx(&allocator, source, JsxLang::Jsx, &JsxCompileConfig {
            ssr: true, ..Default::default()
        });
        json!({
            "id": id, "source": source, "module": output.module_code(), "moduleMap": output.source_map(),
            "components": output.components.iter().map(|component| json!({
                "name": component.component_name(), "setup": format!("{:?}", component.component_setup()),
                "mode": format!("{:?}", component.mode()), "code": component.code(),
                "preamble": component.preamble(), "map": component.map(), "style": format!("{:?}", component.scoped_style()),
            })).collect::<Vec<_>>(),
            "diagnostics": output.diagnostics.iter().map(|diagnostic| json!({
                "message": diagnostic.message.as_str(), "severity": format!("{:?}", diagnostic.severity),
                "start": diagnostic.start, "end": diagnostic.end,
            })).collect::<Vec<_>>(),
        })
    }).collect::<Vec<_>>();
    if let Some(path) = std::env::var_os("VIZE_JSX_FALSY_REFUSAL_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec(&cases).expect("whole refusal packets"),
        )
        .expect("capture refusals");
    }
    for (case, (id, source)) in cases.iter().zip(FIXTURES) {
        assert_eq!(case["module"], "", "{id}: no partial executable module");
        assert_eq!(case["moduleMap"], Value::Null);
        assert_eq!(
            case["components"]
                .as_array()
                .expect("producer packet")
                .len(),
            1
        );
        assert_eq!(
            case["diagnostics"],
            json!([{
                "message": "SSR expression-bodied VDOM components require an explicit functional props and attribute fallthrough projection; use a stateful block-body component or the Vapor component role",
                "severity": "Error", "start": source.find('<').expect("root start"),
                "end": source.rfind('>').expect("root end") + 1,
            }]),
            "{id}"
        );
    }
}

#[test]
fn actual_values_updates_and_hydration_match_jsx_child_semantics() {
    let mut fixtures = compile_fixtures(false);
    fixtures
        .as_object_mut()
        .expect("fixture table")
        .insert("ternary".into(), compile_value_sensitive_ternary());
    if let Some(path) = std::env::var_os("VIZE_JSX_VALUE_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec(&fixtures).expect("whole modules/maps"),
        )
        .expect("capture value-sensitive modules before runtime");
    } else {
        assert_ne!(
            std::env::var("VIZE_L4_SSR_REQUIRE_NATIVE").as_deref(),
            Ok("1"),
            "the affected NativeSSR carrier requires the whole JSX capture"
        );
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
    assert_eq!(receipt["fixtures"], fixtures);
    assert_eq!(
        receipt["ternary"]["phases"]
            .as_array()
            .expect("all value phases")
            .len(),
        15
    );
    assert_eq!(receipt["ternary"]["invertedConditionRefused"], true);
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
            "const flag = count > 0; const alias = flag; let mutable = true; const App = () => <div>{{({condition}) && <span/>}}</div>;"
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
fn same_named_parameter_does_not_inherit_a_boolean_const_proof() {
    let allocator = Allocator::new();
    let source = "const flag = true; const App = (flag) => <div>{flag && <span/>}</div>;";
    let output = lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);
    assert!(!output.has_errors());
    let root = &output.roots[0];
    let TemplateChildNode::Element(element) = &root.root.children[0] else {
        panic!("authored div");
    };
    let TemplateChildNode::For(scope) = &element.children[0] else {
        panic!("parameter's symbol is distinct from the outer boolean const");
    };
    assert!(scope.parse_result.match_scope);
    assert!(root.l2.is_err());
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
