//! Identical before/head observer using only previously published entrypoints.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "measurement packets use ordinary owned strings and JSON fixtures"
)]

use std::{hint::black_box, time::Instant};

use serde_json::json;
use vize_atelier_jsx::{
    JsxLang, SsrCompileOptions, VaporCompileOptions, compile_to_ssr, compile_to_vapor,
};
use vize_l0::{Allocator, profiler::global_profiler};

fn emit(source: &str, ssr: bool) -> Vec<std::string::String> {
    let allocator = Allocator::new();
    if ssr {
        let output = compile_to_ssr(
            &allocator,
            source,
            JsxLang::Tsx,
            SsrCompileOptions::default(),
        );
        assert!(!output.has_errors(), "{:?}", output.diagnostics);
        output
            .components
            .into_iter()
            .map(|component| component.code.to_string())
            .collect()
    } else {
        let output = compile_to_vapor(
            &allocator,
            source,
            JsxLang::Tsx,
            VaporCompileOptions::default(),
        );
        assert!(!output.has_errors(), "{:?}", output.diagnostics);
        output
            .components
            .into_iter()
            .map(|component| component.code.to_string())
            .collect()
    }
}

#[test]
fn capture_default_backend_source_profile() {
    let Some(path) = std::env::var_os("VIZE_JSX_DEFAULT_PROFILE_CAPTURE") else {
        return;
    };
    let profiler = global_profiler();
    profiler.disable();
    let mut rows = Vec::new();
    for source in [
        "const App = () => <p>{label}</p>;",
        "// 日本語 😀\r\nconst App = () => <section><h1>{title}</h1><p id={id}>{label}</p></section>;",
        "const App = () => <ul>{items.map((item: {id:number,label:string}) => <li key={item.id}>{item.label}</li>)}</ul>;",
    ] {
        for ssr in [false, true] {
            let output = emit(source, ssr);
            for _ in 0..30 {
                black_box(emit(black_box(source), ssr));
            }
            let mut samples = Vec::new();
            for _ in 0..9 {
                let started = Instant::now();
                for _ in 0..200 {
                    black_box(emit(black_box(source), ssr));
                }
                samples.push(started.elapsed().as_nanos() as u64);
            }
            profiler.clear();
            profiler.enable();
            let profiled = emit(source, ssr);
            profiler.disable();
            assert_eq!(
                profiled, output,
                "profiling must preserve complete emitted code"
            );
            let phases = profiler
                .summary()
                .entries
                .into_iter()
                .map(|entry| {
                    json!({
                        "name": entry.name, "count": entry.count,
                        "totalNs": entry.total.as_nanos() as u64,
                        "selfNs": entry.self_total.as_nanos() as u64,
                    })
                })
                .collect::<Vec<_>>();
            rows.push(json!({
                "source": source, "ssr": ssr, "output": output,
                "iterationsPerSample": 200, "samplesNs": samples, "phases": phases,
            }));
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
}
