//! Retain the original functional input before/after native module admission.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "complete source qualification packets use owned JSON"
)]
use serde_json::json;
use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, compile_jsx};
use vize_l0::Allocator;

#[test]
fn capture_original_functional_ssr_and_stateful_control_packets() {
    let Some(path) = std::env::var_os("VIZE_JSX_FUNCTIONAL_ROLE_CAPTURE") else {
        return;
    };
    let config = JsxCompileConfig {
        ssr: true,
        compat: JsxCompatMode::Babel,
        ..Default::default()
    };
    let mut packets = Vec::new();
    for source in [
        "const A = () => <B/>;",
        "const A = () => <B v-slots={1}/>;",
        "const A = () => <B v-slots={`text`}/>;",
        "const A = () => { return <B/>; };",
        "const A = () => { return <B v-slots={1}/>; };",
        "const A = () => { return <B v-slots={`text`}/>; };",
    ] {
        let allocator = Allocator::new();
        let output = compile_jsx(&allocator, source, JsxLang::Jsx, &config);
        packets.push(json!({"source":source,"ssr":true,"compat":"babel","module":output.module_code().as_str(),"map":output.source_map(),"components":output.components.iter().map(|component|json!({"code":component.code(),"mode":component.mode().as_config_str(),"map":component.map()})).collect::<Vec<_>>(),"diagnostics":output.diagnostics.iter().map(|diagnostic|json!({"message":diagnostic.message.as_str(),"start":diagnostic.start,"end":diagnostic.end})).collect::<Vec<_>>()}));
    }
    std::fs::write(path, serde_json::to_vec_pretty(&packets).unwrap()).unwrap();
}
