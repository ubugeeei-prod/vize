//! Actual symbol stability controls and complete authored-module/map packets.
#![expect(
    clippy::expect_used,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "complete integration fixture packets"
)]

use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::{Visit, walk};
use oxc_sourcemap::SourceMap;
use oxc_span::{GetSpan, Span};
use serde_json::json;
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx, parse_module};
use vize_l0::{Allocator, profiler::global_profiler};

#[derive(Default)]
struct ComponentCalls(Vec<(String, Span)>);

impl<'a> Visit<'a> for ComponentCalls {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && matches!(
                callee.name.as_str(),
                "_createDynamicComponent" | "_createComponentWithFallback"
            )
            && let Some(value) = call.arguments.first()
        {
            self.0.push((callee.name.to_string(), value.span()));
        }
        walk::walk_call_expression(self, call);
    }
}

fn position(source: &str, offset: usize) -> (u32, u32) {
    let before = source.get(..offset).expect("UTF-8 boundary");
    (
        before.bytes().filter(|byte| *byte == b'\n').count() as u32,
        before
            .rsplit('\n')
            .next()
            .expect("line")
            .encode_utf16()
            .count() as u32,
    )
}

#[test]
fn only_actual_immutable_native_component_symbols_are_static_values() {
    let cases = [
        (
            "const",
            "const Child={}; export const App=()=> <section><Child/></section>;",
            true,
        ),
        (
            "mutable",
            "let Child={}; Child=other; export const App=()=> <section><Child/></section>;",
            false,
        ),
        (
            "import",
            "import Child from './Child.js'; export const App=()=> <section><Child/></section>;",
            false,
        ),
        (
            "namespace",
            "import * as UI from './Child.js'; export const App=()=> <section><UI.Child/></section>;",
            false,
        ),
        (
            "member",
            "const UI={Child:{}}; export const App=()=> <section><UI.Child/></section>;",
            false,
        ),
        (
            "mutable-shadow",
            "const Child={}; export const App=()=> {let Child={}; return <section><Child/></section>;};",
            false,
        ),
        (
            "const-shadow",
            "let Child={}; export const App=()=> {const Child={}; return <section><Child/></section>;};",
            true,
        ),
        (
            "write",
            "const Child={}; Child=other; export const App=()=> <section><Child/></section>;",
            false,
        ),
        (
            "escaped",
            "const \\u0043hild={}; export const App=()=> <section><Child/></section>;",
            true,
        ),
        (
            "unicode-crlf",
            "// 雪 🧭 'quote'\r\nconst 子={};\r\nexport const App=()=> <section><子/></section>;",
            true,
        ),
        (
            "destructured",
            "const {Child}=registry; export const App=()=> <section><Child/></section>;",
            true,
        ),
        (
            "import-shadow",
            "import Child from './Child.js'; export const App=()=> {const Child={}; return <section><Child/></section>;};",
            true,
        ),
    ];
    let profiler = global_profiler();
    profiler.disable();
    let mut packets = Vec::new();
    for (name, source, stable) in cases {
        for mode in [JsxOutputMode::Vapor, JsxOutputMode::Vdom] {
            let allocator = Allocator::new();
            let mut config = JsxCompileConfig {
                default_mode: mode,
                ..Default::default()
            };
            let without = compile_jsx(&allocator, source, JsxLang::Tsx, &config);
            assert!(!without.has_errors(), "{name}: {:?}", without.diagnostics);
            config.vdom.source_map = true;
            profiler.clear();
            profiler.enable();
            let output = compile_jsx(&allocator, source, JsxLang::Tsx, &config);
            profiler.disable();
            assert!(!output.has_errors(), "{name}: {:?}", output.diagnostics);
            let code = output.module_code();
            assert_eq!(
                code,
                without.module_code(),
                "maps do not change code: {name}"
            );
            let parsed = parse_module(allocator.as_oxc(), &code, JsxLang::Tsx);
            assert!(!parsed.has_errors(), "{name}: {:?}", parsed.diagnostics);
            let mut calls = ComponentCalls::default();
            calls.visit_program(&parsed.program);
            let counters = profiler
                .counter_summary()
                .entries
                .into_iter()
                .map(|entry| (entry.name.to_string(), entry.total))
                .collect::<std::collections::BTreeMap<_, _>>();
            let builds = counters
                .get("atelier.jsx.lower.component_stability_builds")
                .copied()
                .unwrap_or(0);
            let proofs = counters
                .get("atelier.jsx.lower.stable_component_values")
                .copied()
                .unwrap_or(0);
            let members = matches!(name, "member" | "namespace");
            if mode == JsxOutputMode::Vapor {
                assert_eq!(
                    builds,
                    u64::from(!members),
                    "one shared semantic fact build: {name}"
                );
                assert_eq!(proofs, u64::from(stable), "actual symbol proof: {name}");
                assert_eq!(calls.0.len(), 1, "one real component creation: {name}");
                let (helper, value) = calls.0.first().expect("component call");
                assert_eq!(
                    helper,
                    if stable {
                        "_createComponentWithFallback"
                    } else {
                        "_createDynamicComponent"
                    }
                );
                if stable {
                    let map = SourceMap::from_json_string(output.source_map().expect("map"))
                        .expect("map JSON");
                    let actual = map
                        .get_source_view_tokens()
                        .map(|token| {
                            (
                                (token.get_dst_line(), token.get_dst_col()),
                                (token.get_src_line(), token.get_src_col()),
                            )
                        })
                        .collect::<Vec<_>>();
                    let tag = if name == "unicode-crlf" {
                        "子"
                    } else {
                        "Child"
                    };
                    let authored = source
                        .rfind(&format!("<{tag}/>"))
                        .expect("authored component tag")
                        + 1;
                    let origin = (
                        position(&code, value.start as usize),
                        position(source, authored),
                    );
                    assert_eq!(
                        actual
                            .iter()
                            .filter(|token| token.0 == origin.0)
                            .copied()
                            .collect::<Vec<_>>(),
                        [origin],
                        "actual component-value origin: {name}"
                    );
                }
            } else {
                assert_eq!(builds, 0, "default VDOM performs no stability scan: {name}");
                assert_eq!(proofs, 0);
                assert_eq!(calls.0, []);
            }
            let map =
                SourceMap::from_json_string(output.source_map().expect("map")).expect("map JSON");
            assert_eq!(
                map.get_source_contents().collect::<Vec<_>>(),
                [Some(source)]
            );
            let tokens = map
                .get_source_view_tokens()
                .map(|token| {
                    (
                        (token.get_dst_line(), token.get_dst_col()),
                        (token.get_src_line(), token.get_src_col()),
                    )
                })
                .collect::<Vec<_>>();
            packets.push(json!({"name":name,"source":source,"mode":format!("{mode:?}"),"stable":stable,"code":code.as_str(),"map":serde_json::from_str::<serde_json::Value>(output.source_map().expect("map")).expect("map JSON"),"tokens":tokens,"counters":counters}));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_JSX_COMPONENT_STABILITY_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).expect("packets"))
            .expect("capture");
    }
    let expected: serde_json::Value = serde_json::from_str(include_str!("../../../tests/_fixtures/differential/jsx/authored-native-modules-10700/component-stability.json")).expect("reviewed whole packets");
    assert_eq!(json!(packets), expected);
}
