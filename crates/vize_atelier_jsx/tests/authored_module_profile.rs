//! Same-source observer for the old refusal and new authored-module contracts.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "measurement packets use owned JSON fixtures"
)]
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_l0::{Allocator, profiler::global_profiler};

fn emit(source: &str, ssr: bool, map: bool) -> Value {
    let allocator = Allocator::new();
    let mut config = JsxCompileConfig {
        ssr,
        default_mode: JsxOutputMode::Vapor,
        ..Default::default()
    };
    config.vdom.source_map = map;
    let output = compile_jsx(&allocator, source, JsxLang::Tsx, &config);
    json!({"code":output.module_code().as_str(),"map":output.source_map(),"diagnostics":output.diagnostics.iter().map(|diagnostic| json!({"message":diagnostic.message.as_str(),"start":diagnostic.start,"end":diagnostic.end})).collect::<Vec<_>>()})
}

#[test]
fn capture_authored_module_source_profile() {
    let Some(path) = std::env::var_os("VIZE_JSX_AUTHORED_PROFILE_CAPTURE") else {
        return;
    };
    let profiler = global_profiler();
    profiler.disable();
    let mut rows = Vec::new();
    for source in [
        "import {ref} from 'vue'; const label=ref('text'); export const App=()=> <p>{label.value}</p>; export default App;",
        "// 日本語 😀\r\nimport Child from './Child.js'; export const App=(props:{label:string})=> {const label=props.label; return <Child label={label}/>;}; export default App;",
        "const init=sideEffect(); export const A=()=> <p>{init}</p>; export const B=()=> <b>{init}</b>; export default A;",
    ] {
        for ssr in [false, true] {
            for map in [false, true] {
                let output = emit(source, ssr, map);
                for _ in 0..20 {
                    black_box(emit(black_box(source), ssr, map));
                }
                let mut samples = Vec::new();
                for _ in 0..9 {
                    let start = Instant::now();
                    for _ in 0..100 {
                        black_box(emit(black_box(source), ssr, map));
                    }
                    samples.push(start.elapsed().as_nanos() as u64);
                }
                profiler.clear();
                profiler.enable();
                let observed = emit(source, ssr, map);
                profiler.disable();
                assert_eq!(
                    observed, output,
                    "source profiling cannot change complete outputs"
                );
                let counters = profiler.counter_summary().entries.into_iter().map(|entry| json!({"name":entry.name,"samples":entry.samples,"total":entry.total})).collect::<Vec<_>>();
                let summary = profiler.summary();
                let phases=summary.entries.into_iter().map(|entry|json!({"name":entry.name,"count":entry.count,"totalNs":entry.total.as_nanos() as u64,"selfNs":entry.self_total.as_nanos() as u64})).collect::<Vec<_>>();
                rows.push(json!({"source":source,"ssr":ssr,"map":map,"output":output,"iterationsPerSample":100,"samplesNs":samples,"phases":phases,"counters":counters}));
            }
        }
    }
    std::fs::write(path, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
}
