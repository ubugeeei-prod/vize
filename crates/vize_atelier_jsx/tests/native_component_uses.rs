//! Converted component values keep a proved role and an actual binding owner.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "exact source/profile fixtures use owned JSON"
)]
use serde_json::json;
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_l0::{Allocator, profiler::global_profiler};

const REFUSAL: &str = "Vapor/SSR component-object bindings may be used only as JSX component tags or direct exports; local calls, callable aliases, introspection and value escapes require an explicit callable representation";

fn counters() -> Vec<(&'static str, u64)> {
    let mut entries = global_profiler()
        .counter_summary()
        .entries
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.name,
                "atelier.jsx.module.component_use_candidates"
                    | "atelier.jsx.module.component_use_semantic_builds"
                    | "atelier.jsx.module.component_use_refusals"
            )
        })
        .map(|entry| (entry.name, entry.total))
        .collect::<Vec<_>>();
    entries.sort_unstable_by_key(|entry| entry.0);
    entries
}

#[test]
fn callable_alias_introspection_and_escape_uses_refuse_actual_converted_symbols() {
    let profiler = global_profiler();
    let mut packets = Vec::new();
    let declaration = "// 日本語 😀\r\nconst App = () => <p>Hello</p>;\r\n";
    for ssr in [false, true] {
        for suffix in [
            "export const node = App();",
            "export const node = App.call(null);",
            "export const node = App.apply(null, []);",
            "export const Bound = App.bind(null);",
            "const Alias = App; export const node = Alias();",
            "const box = [App]; export const node = box[0]();",
            "const box = { component: App }; export const node = box.component();",
            "let Alias; Alias = App; export const node = Alias();",
            "export const shape = typeof App;",
            "export const name = App.name;",
            "export const registry = { App };",
            "consume(App);",
            "type Shape = typeof App;",
        ] {
            let source = format!("{declaration}{suffix}");
            let allocator = Allocator::new();
            profiler.clear();
            profiler.enable();
            let output = compile_jsx(
                &allocator,
                &source,
                JsxLang::Tsx,
                &JsxCompileConfig {
                    ssr,
                    default_mode: JsxOutputMode::Vapor,
                    ..Default::default()
                },
            );
            profiler.disable();
            assert_eq!(
                output
                    .diagnostics
                    .iter()
                    .map(|diagnostic| diagnostic.message.as_str())
                    .collect::<Vec<_>>(),
                [REFUSAL],
                "{source}"
            );
            let diagnostic = output.diagnostics.first().unwrap();
            assert_eq!(
                source.get(diagnostic.start as usize..diagnostic.end as usize),
                Some("App")
            );
            assert_eq!(output.module_code().as_str(), "");
            assert_eq!(
                counters(),
                [
                    ("atelier.jsx.module.component_use_candidates", 1),
                    ("atelier.jsx.module.component_use_refusals", 1),
                    ("atelier.jsx.module.component_use_semantic_builds", 1),
                ]
            );
            packets.push(json!({"source":source,"ssr":ssr,"module":output.module_code().as_str(),"diagnostics":output.diagnostics.iter().map(|diagnostic|json!({"message":diagnostic.message.as_str(),"start":diagnostic.start,"end":diagnostic.end})).collect::<Vec<_>>(),"counters":counters()}));
            // The unchanged VDOM functional lane retains the exact local value
            // use and its callable representation instead of acquiring this guard.
            let vdom = compile_jsx(
                &allocator,
                &source,
                JsxLang::Tsx,
                &JsxCompileConfig::default(),
            );
            assert_eq!(
                vdom.diagnostics.len(),
                0,
                "{source}: {:?}",
                vdom.diagnostics
            );
        }
        for (suffix, candidates, builds) in [
            (
                "export {App as Alias}; export default App; export const Parent = () => <App/>;",
                0,
                0,
            ),
            (
                "function ordinary(App: () => null) { return App(); } export default App;",
                1,
                1,
            ),
            (
                "function ordinary(App: any) { return App.call(null); } export default App;",
                1,
                1,
            ),
            ("export default App as unknown;", 0, 0),
        ] {
            let source = format!("{declaration}{suffix}");
            let allocator = Allocator::new();
            profiler.clear();
            profiler.enable();
            let output = compile_jsx(
                &allocator,
                &source,
                JsxLang::Tsx,
                &JsxCompileConfig {
                    ssr,
                    default_mode: JsxOutputMode::Vapor,
                    ..Default::default()
                },
            );
            profiler.disable();
            assert_eq!(
                output.diagnostics.len(),
                0,
                "{source}: {:?}",
                output.diagnostics
            );
            assert_eq!(
                counters(),
                [
                    ("atelier.jsx.module.component_use_candidates", candidates),
                    ("atelier.jsx.module.component_use_refusals", 0),
                    ("atelier.jsx.module.component_use_semantic_builds", builds),
                ]
            );
            packets.push(json!({"source":source,"ssr":ssr,"module":output.module_code().as_str(),"diagnostics":[],"counters":counters()}));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_JSX_NATIVE_USES_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
    }
}
