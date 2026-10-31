//! Preserve complete genuine before/head producer packets for slot and comma-expression reviews.
#![expect(
    clippy::expect_used,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "hosted integration observer retains whole source and map packets"
)]
use oxc_sourcemap::SourceMap;
use serde_json::{Value, json};
use std::path::PathBuf;
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, compile_jsx, parse_module};
use vize_l0::Allocator;

fn position(source: &str, offset: usize) -> (u32, u32) {
    let before = &source[..offset];
    (
        before.bytes().filter(|b| *b == b'\n').count() as u32,
        before
            .rsplit('\n')
            .next()
            .expect("line")
            .encode_utf16()
            .count() as u32,
    )
}

#[test]
fn complete_review_sources_and_genuine_authored_maps() {
    let original: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json"
    ))
    .expect("unchanged original");
    let mut cases = Vec::new();
    for (kind, body) in [
        (
            "sequence",
            "const App=()=> <main>{state.record(\"left\"),state.record(\"right\"),state.value}</main>;",
        ),
        (
            "slot",
            "import {defineComponent,renderSlot} from \"vue\";\nconst Host=defineComponent({setup(_props,{slots}){return()=>renderSlot(slots,\"default\")}});\nconst App=()=> <Host>{{default:()=> <main>{state.value}</main>}}</Host>;",
        ),
    ] {
        let source = format!("// 日本語 雪🌸 'quoted'\r\n{body}");
        for mapped in [false, true] {
            let allocator = Allocator::new();
            let mut options = JsxCompileConfig::default();
            options.vdom.source_map = mapped;
            let output = compile_jsx(&allocator, &source, JsxLang::Jsx, &options);
            let code = output.module_code();
            let mut tokens = Vec::new();
            if let Some(encoded) = output.source_map() {
                let map = SourceMap::from_json_string(encoded).expect("actual generated map");
                assert_eq!(
                    map.get_source_contents().collect::<Vec<_>>(),
                    [Some(source.as_str())]
                );
                tokens = map
                    .get_source_view_tokens()
                    .map(|token| {
                        (
                            (token.get_dst_line(), token.get_dst_col()),
                            (token.get_src_line(), token.get_src_col()),
                        )
                    })
                    .collect();
            }
            cases.push(json!({
                "kind":kind,"source":source,"mapped":mapped,"code":code,"functionName":"App",
                "map":output.source_map(),"tokens":tokens,
                "diagnostics":output.diagnostics.iter().map(|d| json!({"message":d.message,"start":d.start,"end":d.end,"error":d.is_error()})).collect::<Vec<_>>(),
                "components":output.components.iter().map(|c| json!({"name":c.component_name(),"mode":format!("{:?}",c.mode()),"code":c.code(),"preamble":c.preamble(),"map":c.map()})).collect::<Vec<_>>()
            }));
        }
    }
    let packet = json!({"original":original,"cases":cases});
    if let Some(path) = std::env::var_os("VIZE_JSX_VDOM_REVIEW_CAPTURE").map(PathBuf::from) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("capture parent");
        }
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&packet).expect("whole JSON"),
        )
        .expect("preserve before validation");
    }
    for case in packet["cases"].as_array().expect("cases") {
        assert_eq!(case["diagnostics"], json!([]));
        let source = case["source"].as_str().expect("source");
        let code = case["code"].as_str().expect("code");
        let allocator = Allocator::new();
        assert!(
            !parse_module(allocator.as_oxc(), code, JsxLang::Jsx).has_errors(),
            "{code}"
        );
        if case["mapped"] == true {
            let generated = code.rfind("state.value").expect("actual expression");
            let authored = source.rfind("state.value").expect("authored expression");
            assert!(
                case["tokens"]
                    .as_array()
                    .expect("map tokens")
                    .contains(&json!([
                        position(code, generated),
                        position(source, authored)
                    ])),
                "authored expression byte ownership is retained: {case}"
            );
        }
    }
    for pair in packet["cases"].as_array().expect("cases").chunks_exact(2) {
        assert_eq!(
            pair[0]["code"], pair[1]["code"],
            "map selection cannot change runtime semantics"
        );
    }
}
