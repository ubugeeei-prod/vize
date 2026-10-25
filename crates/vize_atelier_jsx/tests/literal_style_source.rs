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
    let mut rows = Vec::new();
    for (id, expression, expected) in [
        ("font", "{fontWeight:'700'}", Some("font-weight:700;")),
        (
            "duplicates",
            "{color:'red',fontWeight:'700',color:'blue'}",
            Some("color:blue;font-weight:700;"),
        ),
        (
            "custom",
            "{'--Brand':'blue',fontWeight:'700'}",
            Some("--Brand:blue;font-weight:700;"),
        ),
        ("call", "{color:read()}", None),
        ("reference", "{color:shade}", None),
        ("spread", "{...other,color:'red'}", None),
        ("computed", "{['color']:'red'}", None),
        ("getter", "{get color(){return read()}}", None),
        ("numeric", "{fontWeight:700}", None),
        ("empty", "{}", None),
        ("empty-value", "{color:''}", None),
        ("declaration", "{color:'red;display:none'}", None),
    ] {
        for (prefix, ending) in [("", "\n"), ("<!-- 日本語 😀 -->\r\n", "\r\n")] {
            let source = format!(
                "{prefix}<button type=\"button\" :style=\"{expression}\" id=\"own\">{{{{label}}}}</button>{ending}"
            );
            for retained in [false, true] {
                for mapped in [false, true] {
                    profiler.disable();
                    let (output, arena, reparses) = template(&source, retained, mapped);
                    assert_eq!(output["errors"], json!([]), "{id}: {output}");
                    if !old && let Some(css) = expected {
                        let static_source = source.replace(
                            &format!(":style=\"{expression}\""),
                            &format!("style=\"{css}\""),
                        );
                        let (reference, _, _) = template(&static_source, retained, mapped);
                        assert_eq!(
                            output["code"], reference["code"],
                            "whole static template code: {id}/{retained}/{mapped}"
                        );
                        assert_eq!(output["templates"], reference["templates"]);
                    }
                    profiler.clear();
                    profiler.enable();
                    let (profiled, _, _) = template(&source, retained, mapped);
                    profiler.disable();
                    assert_eq!(output, profiled);
                    let counters = profiler
                        .counter_summary()
                        .entries
                        .into_iter()
                        .map(|c| json!({"name":c.name,"total":c.total}))
                        .collect::<Vec<_>>();
                    let phases = profiler.summary().entries.into_iter().map(|p|json!({"name":p.name,"count":p.count,"totalNs":p.total.as_nanos() as u64})).collect::<Vec<_>>();
                    rows.push(json!({"id":id,"family":"template","source":source,"retained":retained,"mapped":mapped,"fold":expected.is_some(),"output":output,"arenaBytes":arena,"expressionReparses":reparses,"counters":counters,"phases":phases}));
                }
            }
        }
        let source = format!(
            "// 日本語 😀\r\nconst App = () => <button type=\"button\" style={{{expression}}} id=\"own\">{{label}}</button>;\r\n"
        );
        for (mode, ssr, standalone) in [
            (JsxOutputMode::Vapor, false, false),
            (JsxOutputMode::Vdom, false, false),
            (JsxOutputMode::Vapor, true, false),
            (JsxOutputMode::Vapor, false, true),
        ] {
            profiler.disable();
            let (output, arena, reparses) = jsx(&source, mode, ssr, standalone);
            if !old
                && !ssr
                && mode == JsxOutputMode::Vapor
                && let Some(css) = expected
            {
                let static_source = source.replace(
                    &format!("style={{{expression}}}"),
                    &format!("style=\"{css}\""),
                );
                let (reference, _, _) = jsx(&static_source, mode, ssr, standalone);
                assert_eq!(
                    output["components"], reference["components"],
                    "whole JSX static component packet: {id}/{standalone}"
                );
            }
            profiler.clear();
            profiler.enable();
            let (profiled, _, _) = jsx(&source, mode, ssr, standalone);
            profiler.disable();
            assert_eq!(output, profiled);
            let counters = profiler
                .counter_summary()
                .entries
                .into_iter()
                .map(|c| json!({"name":c.name,"total":c.total}))
                .collect::<Vec<_>>();
            rows.push(json!({"id":id,"family":"jsx","source":source,"mode":format!("{mode:?}"),"ssr":ssr,"standalone":standalone,"fold":expected.is_some() && !ssr && mode == JsxOutputMode::Vapor,"output":output,"arenaBytes":arena,"expressionReparses":reparses,"counters":counters}));
        }
    }
    // A pure value alone does not prove how authored style sources merge.
    // These complete packets must remain byte-identical to the old runtime
    // path, including authored order, effects, diagnostics and every map.
    for (id, attributes) in [
        (
            "static-before",
            "style=\"color:red\" :style=\"{fontWeight:'700'}\"",
        ),
        (
            "static-after",
            ":style=\"{fontWeight:'700'}\" style=\"color:red\"",
        ),
        (
            "spread-before",
            "v-bind=\"attrs\" :style=\"{fontWeight:'700'}\"",
        ),
        (
            "spread-after",
            ":style=\"{fontWeight:'700'}\" v-bind=\"attrs\"",
        ),
        (
            "computed-name",
            ":[key]=\"value\" :style=\"{fontWeight:'700'}\"",
        ),
        ("prop-modifier", ":style.prop=\"{fontWeight:'700'}\""),
        ("attr-modifier", ":style.attr=\"{fontWeight:'700'}\""),
    ] {
        let source = format!("<button {attributes}>{{{{label}}}}</button>");
        for retained in [false, true] {
            for mapped in [false, true] {
                let (output, arena, reparses) = template(&source, retained, mapped);
                rows.push(json!({"id":id,"family":"template-merge","source":source,"retained":retained,"mapped":mapped,"fold":false,"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
            }
        }
    }
    for (id, opening) in [
        (
            "spread-before",
            "button {...attrs} style={{fontWeight:'700'}}",
        ),
        (
            "spread-after",
            "button style={{fontWeight:'700'}} {...attrs}",
        ),
        (
            "duplicate-style",
            "button style=\"color:red\" style={{fontWeight:'700'}}",
        ),
        ("component-prop", "Child style={{fontWeight:'700'}}"),
    ] {
        let closing = if id == "component-prop" {
            "Child"
        } else {
            "button"
        };
        let source = format!("const App = () => <{opening}>{{label}}</{closing}>;");
        for standalone in [false, true] {
            let (output, arena, reparses) = jsx(&source, JsxOutputMode::Vapor, false, standalone);
            rows.push(json!({"id":id,"family":"jsx-merge","source":source,"standalone":standalone,"fold":false,"output":output,"arenaBytes":arena,"expressionReparses":reparses}));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_LITERAL_STYLE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
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
