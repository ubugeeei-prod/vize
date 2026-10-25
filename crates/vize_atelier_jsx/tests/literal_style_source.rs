//! Before/head observer uses only existing public compiler entrypoints.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "complete measurement packets"
)]

use serde_json::{Value, json};
use vize_atelier_jsx::{
    JsxCompileConfig, JsxLang, JsxOutputMode, VaporCompileOptions, compile_jsx, compile_to_vapor,
};
use vize_atelier_vapor::{
    VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor_with_experimental_options,
};
use vize_l0::{Allocator, profiler::global_profiler};

fn jsx(source: &str, mode: JsxOutputMode, ssr: bool, standalone: bool) -> (Value, usize, u64) {
    let before = vize_atelier_core::expr_parse_probe::expr_parse_count();
    let allocator = Allocator::new();
    let output = if standalone {
        let result = compile_to_vapor(
            &allocator,
            source,
            JsxLang::Tsx,
            VaporCompileOptions { ssr },
        );
        json!({"components":result.components.into_iter().map(|c|json!({"code":c.code,"templates":c.templates})).collect::<Vec<_>>(),"errors":result.diagnostics.iter().map(|d|format!("{d:?}")).collect::<Vec<_>>()})
    } else {
        let result = compile_jsx(
            &allocator,
            source,
            JsxLang::Tsx,
            &JsxCompileConfig {
                default_mode: mode,
                ssr,
                ..Default::default()
            },
        );
        json!({"module":result.module_code(),"map":result.source_map(),"components":result.components.iter().map(|c|json!({"code":c.code(),"preamble":c.preamble(),"map":c.map()})).collect::<Vec<_>>(),"errors":result.diagnostics.iter().map(|d|format!("{d:?}")).collect::<Vec<_>>()})
    };
    (
        output,
        allocator.allocated_bytes(),
        vize_atelier_core::expr_parse_probe::expr_parse_count() - before,
    )
}

fn template(source: &str, retained: bool, mapped: bool) -> (Value, usize, u64) {
    let before = vize_atelier_core::expr_parse_probe::expr_parse_count();
    let allocator = Allocator::new();
    let result = compile_vapor_with_experimental_options(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane: retained,
            ..Default::default()
        },
        VaporCompilerExperimentalOptions {
            source_map: mapped,
            source_map_filename: Some("Original 日😀.vue".into()),
            ..Default::default()
        },
    );
    (
        json!({"code":result.code,"templates":result.templates,"map":result.map,"errors":result.error_messages,"warnings":result.warning_messages}),
        allocator.allocated_bytes(),
        vize_atelier_core::expr_parse_probe::expr_parse_count() - before,
    )
}

#[test]
fn complete_literal_style_source_observations() {
    let old = std::env::var_os("VIZE_LITERAL_STYLE_EXPECT_OLD").is_some();
    let profiler = global_profiler();
    let inputs: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/inputs.json"
    )).unwrap();
    assert_eq!(inputs.len(), 180);
    let mut rows = Vec::new();
    for mut row in inputs {
        let source = row["source"].as_str().unwrap();
        profiler.disable();
        let emit = |source: &str| match row["family"].as_str().unwrap() {
            "template" | "template-merge" => template(
                source,
                row["retained"].as_bool().unwrap(),
                row["mapped"].as_bool().unwrap(),
            ),
            "jsx" | "jsx-merge" => jsx(
                source,
                if row["mode"].as_str() == Some("Vdom") {
                    JsxOutputMode::Vdom
                } else {
                    JsxOutputMode::Vapor
                },
                row["ssr"].as_bool().unwrap_or(false),
                row["standalone"].as_bool().unwrap(),
            ),
            _ => panic!("unregistered source fixture family"),
        };
        let (output, arena, reparses) = emit(source);
        if !old && row["fold"] == true {
            let (reference, _, _) = emit(row["referenceSource"].as_str().unwrap());
            if row["family"] == "template" {
                assert_eq!(
                    output["code"], reference["code"],
                    "whole static template code: {row}"
                );
                assert_eq!(output["templates"], reference["templates"]);
                assert_eq!(output["errors"], json!([]));
            } else {
                // The current-main mode-aware module boundary may still refuse
                // native modules; that actual diagnostic packet is retained.
                // This slice qualifies complete component output only.
                assert_eq!(
                    output["components"], reference["components"],
                    "whole JSX static component packet: {row}"
                );
            }
        }
        profiler.clear();
        profiler.enable();
        let (profiled, _, _) = emit(source);
        profiler.disable();
        assert_eq!(output, profiled);
        let counters = profiler
            .counter_summary()
            .entries
            .into_iter()
            .map(|c| json!({"name":c.name,"total":c.total}))
            .collect::<Vec<_>>();
        let phases = profiler
            .summary()
            .entries
            .into_iter()
            .map(|p| json!({"name":p.name,"count":p.count,"totalNs":p.total.as_nanos() as u64}))
            .collect::<Vec<_>>();
        row["output"] = output;
        row["arenaBytes"] = json!(arena);
        row["expressionReparses"] = json!(reparses);
        row["counters"] = json!(counters);
        row["phases"] = json!(phases);
        rows.push(row);
    }
    if let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}

#[test]
fn reviewed_cssom_boundaries_keep_the_original_runtime_operations() {
    let old = std::env::var_os("VIZE_LITERAL_STYLE_EXPECT_OLD").is_some();
    let inputs:Vec<Value>=serde_json::from_str(include_str!("../../../tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/review-inputs.json")).unwrap();
    assert_eq!(inputs.len(), 8);
    let mut packets = Vec::new();
    for input in inputs {
        let expression = input["expression"].as_str().unwrap();
        for standalone in [false, true] {
            let source =
                format!("export const App = () => <button style={{{expression}}}>ready</button>;");
            let (output, arena, reparses) = jsx(&source, JsxOutputMode::Vapor, false, standalone);
            packets.push(json!({"id":input["id"],"family":"jsx","source":source,"standalone":standalone,"fold":input["fold"],"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
        }
        let encoded = expression
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let source = format!("<button :style=\"{encoded}\">ready</button>");
        for retained in [false, true] {
            for mapped in [false, true] {
                let (output, arena, reparses) = template(&source, retained, mapped);
                assert_eq!(output["errors"], json!([]));
                if !old && input["fold"] == true {
                    let css = input["referenceCss"].as_str().unwrap();
                    let static_source = format!("<button style=\"{css}\">ready</button>");
                    assert_eq!(
                        output["code"],
                        template(&static_source, retained, mapped).0["code"]
                    );
                }
                packets.push(json!({"id":input["id"],"family":"template","source":source,"retained":retained,"mapped":mapped,"fold":input["fold"],"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
            }
        }
    }
    if let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_REVIEW_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}

#[test]
fn priority_tokens_keep_original_runtime_semantics() {
    let old = std::env::var_os("VIZE_LITERAL_STYLE_EXPECT_OLD").is_some();
    let inputs:Vec<Value>=serde_json::from_str(include_str!("../../../tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/priority-inputs.json")).unwrap();
    assert_eq!(inputs.len(), 8);
    let mut packets = Vec::new();
    for input in inputs {
        let expression = input["expression"].as_str().unwrap();
        for standalone in [false, true] {
            let source =
                format!("export const App = () => <button style={{{expression}}}>ready</button>;");
            let (output, arena, reparses) = jsx(&source, JsxOutputMode::Vapor, false, standalone);
            packets.push(json!({"id":input["id"],"family":"jsx","source":source,"standalone":standalone,"fold":input["fold"],"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
        }
        let encoded = expression
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let source = format!("<button :style=\"{encoded}\">ready</button>");
        for retained in [false, true] {
            for mapped in [false, true] {
                let (output, arena, reparses) = template(&source, retained, mapped);
                assert_eq!(output["errors"], json!([]));
                if !old && input["fold"] == true {
                    let css = input["referenceCss"].as_str().unwrap();
                    let static_source = format!("<button style=\"{css}\">ready</button>");
                    assert_eq!(
                        output["code"],
                        template(&static_source, retained, mapped).0["code"]
                    );
                }
                packets.push(json!({"id":input["id"],"family":"template","source":source,"retained":retained,"mapped":mapped,"fold":input["fold"],"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
            }
        }
    }
    if let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_PRIORITY_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}

#[test]
fn case_equivalent_style_sources_keep_the_complete_runtime_path() {
    let inputs:Vec<Value>=serde_json::from_str(include_str!("../../../tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/attribute-case-inputs.json")).unwrap();
    assert_eq!(inputs.len(), 8);
    let mut packets = Vec::new();
    for input in inputs {
        for retained in [false, true] {
            for mapped in [false, true] {
                let source = input["template"].as_str().unwrap();
                let (output, arena, reparses) = template(source, retained, mapped);
                packets.push(json!({"id":input["id"],"family":"template","source":source,"retained":retained,"mapped":mapped,"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
            }
        }
        for standalone in [false, true] {
            let source = input["jsx"].as_str().unwrap();
            let (output, arena, reparses) = jsx(source, JsxOutputMode::Vapor, false, standalone);
            packets.push(json!({"id":input["id"],"family":"jsx","source":source,"standalone":standalone,"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_CASE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}

#[test]
fn measured_whole_compile_source_work() {
    let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_PROFILE_CAPTURE") else {
        return;
    };
    let profiler = global_profiler();
    profiler.disable();
    let mut rows = Vec::new();
    for (id, expression) in [
        ("literal", "{fontWeight:'700'}"),
        ("call", "{color:read()}"),
        ("priority-safe", "{color:'red !important',fontWeight:'700'}"),
        (
            "priority-refused",
            "{color:'red !IMPORTANT',fontWeight:'700'}",
        ),
    ] {
        let source =
            format!("const App = () => <button style={{{expression}}}>{{label}}</button>;");
        for (mode, ssr, standalone) in [
            (JsxOutputMode::Vapor, false, false),
            (JsxOutputMode::Vdom, false, false),
            (JsxOutputMode::Vapor, true, false),
            (JsxOutputMode::Vapor, false, true),
        ] {
            let mut samples = Vec::new();
            for _ in 0..10 {
                std::hint::black_box(jsx(&source, mode, ssr, standalone));
            }
            for _ in 0..9 {
                let started = std::time::Instant::now();
                for _ in 0..100 {
                    std::hint::black_box(jsx(std::hint::black_box(&source), mode, ssr, standalone));
                }
                samples.push(started.elapsed().as_nanos() as u64);
            }
            let (output, arena, reparses) = jsx(&source, mode, ssr, standalone);
            rows.push(json!({"id":id,"family":"jsx","source":source,"mode":format!("{mode:?}"),"ssr":ssr,"standalone":standalone,"output":output,"samplesNs":samples,"iterationsPerSample":100,"arenaBytes":arena,"expressionReparses":reparses}));
        }
        let source = format!("<button :style=\"{expression}\">{{{{label}}}}</button>");
        for retained in [false, true] {
            let mut samples = Vec::new();
            for _ in 0..10 {
                std::hint::black_box(template(&source, retained, false));
            }
            for _ in 0..9 {
                let started = std::time::Instant::now();
                for _ in 0..100 {
                    std::hint::black_box(template(std::hint::black_box(&source), retained, false));
                }
                samples.push(started.elapsed().as_nanos() as u64);
            }
            let (output, arena, reparses) = template(&source, retained, false);
            rows.push(json!({"id":id,"family":"template","source":source,"retained":retained,"output":output,"samplesNs":samples,"iterationsPerSample":100,"arenaBytes":arena,"expressionReparses":reparses}));
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
}
