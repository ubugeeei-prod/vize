//! Complete authored-module packets and independent lexical source-map oracles.
#![expect(
    clippy::expect_used,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "integration assertions retain complete JSON packets"
)]
use oxc_ast::ast::{Expression, VariableDeclarator};
use oxc_ast_visit::{Visit, walk};
use oxc_sourcemap::SourceMap;
use serde_json::json;
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx, parse_module};
use vize_l0::Allocator;

fn position(source: &str, offset: usize) -> (u32, u32) {
    let before = source.get(..offset).expect("UTF-8 boundary");
    let line = before.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let column = before
        .rsplit('\n')
        .next()
        .expect("line")
        .encode_utf16()
        .count() as u32;
    (line, column)
}

#[test]
fn owned_native_arrows_keep_whole_modules_and_exact_composed_origins() {
    let mut packets = Vec::new();
    for prefix in [
        "// before\n",
        "// 日本語 😀 'quoted'\r\n",
        "// earlier edit\n// 日本語 😀\n",
    ] {
        for ssr in [false, true] {
            let source = format!(
                "{prefix}import {{ ref }} from 'vue';\nconst label = ref('value');\nexport const App = () => <p>{{label.value}}</p>;\nexport {{ App as Named }};\nexport default App;\n"
            );
            let allocator = Allocator::new();
            let mut options = JsxCompileConfig {
                ssr,
                default_mode: JsxOutputMode::Vapor,
                ..Default::default()
            };
            let without = compile_jsx(&allocator, &source, JsxLang::Tsx, &options);
            assert!(!without.has_errors(), "{:?}", without.diagnostics);
            options.vdom.source_map = true;
            let with = compile_jsx(&allocator, &source, JsxLang::Tsx, &options);
            assert!(!with.has_errors(), "{:?}", with.diagnostics);
            let code = with.module_code();
            assert_eq!(
                without.module_code(),
                code,
                "maps must not change generated code"
            );
            assert!(
                !parse_module(allocator.as_oxc(), &code, JsxLang::Tsx).has_errors(),
                "{code}"
            );
            let map = SourceMap::from_json_string(with.source_map().expect("native module map"))
                .expect("valid map");
            assert_eq!(
                map.get_source_contents().collect::<Vec<_>>(),
                [Some(source.as_str())]
            );
            let actual: Vec<_> = map
                .get_source_view_tokens()
                .map(|token| {
                    assert_eq!(token.get_name(), None);
                    (
                        (token.get_dst_line(), token.get_dst_col()),
                        (token.get_src_line(), token.get_src_col()),
                    )
                })
                .collect();
            // All retained authored bytes plus the two genuine backend links.
            // Component wrappers and generated imports have no fabricated links.
            let arrow = source.find("() =>").expect("arrow");
            let end = source.find("</p>").expect("closing root") + 4;
            let before = source.get(..arrow).expect("prefix");
            let after = source.get(end..).expect("suffix");
            let before_generated = code.find(before).expect("whole authored prefix");
            let after_generated = code.rfind(after).expect("whole authored suffix");
            let mut expected = Vec::new();
            for (original_start, generated_start, text) in
                [(0, before_generated, before), (end, after_generated, after)]
            {
                expected.extend(text.char_indices().map(|(offset, _)| {
                    (
                        position(&code, generated_start + offset),
                        position(&source, original_start + offset),
                    )
                }));
            }
            let authored_tag = source.find("<p>").expect("root") + 1;
            let generated_tag = if ssr {
                code.find("`<p").expect("SSR tag") + 2
            } else {
                code.find("\"<p").expect("Vapor template") + 2
            };
            expected.push((
                position(&code, generated_tag),
                position(&source, authored_tag),
            ));
            let authored_label = source.find("{label.value}").expect("expression") + 1;
            let generated_label = code.rfind("label.value").expect("emitted closure capture");
            expected.push((
                position(&code, generated_label),
                position(&source, authored_label),
            ));
            expected.sort_unstable_by_key(|entry| entry.0);
            assert_eq!(actual, expected, "complete decoded token array: {code}");
            packets.push(json!({"source":source,"ssr":ssr,"code":code.as_str(),"map":serde_json::from_str::<serde_json::Value>(with.source_map().expect("map")).expect("JSON"),"tokens":actual}));
        }
    }
    if let Some(path) = std::env::var_os("VIZE_JSX_AUTHORED_NATIVE_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packets).expect("packets"))
            .expect("capture");
    }
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/jsx/authored-native-modules-10700/mapped-roots.json"
    ))
    .expect("frozen complete source packet");
    let whole_outputs = packets
        .into_iter()
        .map(|packet| {
            let mut fields = packet.as_object().expect("whole output object").clone();
            fields.remove("tokens"); // Independently verified above and retained in raw capture.
            serde_json::Value::Object(fields)
        })
        .collect::<Vec<_>>();
    assert_eq!(json!(whole_outputs), expected);
}

#[test]
fn native_module_admission_is_owned_by_ast_not_enclosing_name_spelling() {
    let mut packets = Vec::new();
    let cases = [
        "export default () => <strong>Hello</strong>;",
        "export const App = () => <div>{flag ? <i>yes</i> : null}</div>;",
        "import Child from './Child.js'; export const App = () => <Child/>; export default App;",
        "import * as UI from './Child.js'; export const App = () => <UI.Child/>;",
        "import {ref} from 'vue'; export const Counter = (props: {label:string}) => { const clicks=ref(0); const inc=()=>clicks.value++; return <button onClick={inc}>{props.label}:{clicks.value}</button>; }; export default Counter;",
        "const mark=sideEffect(); export const A=()=> <i>{mark}</i>; export const B=()=> <b>{mark}</b>; export {A as Alias}; export default B;",
        "#!/usr/bin/env node\n'use client';\nexport default () => <p>Hello</p>;",
    ];
    for ssr in [false, true] {
        for source in cases {
            let allocator = Allocator::new();
            let mut config = JsxCompileConfig {
                ssr,
                default_mode: JsxOutputMode::Vapor,
                ..Default::default()
            };
            config.vdom.source_map = true;
            let out = compile_jsx(&allocator, source, JsxLang::Tsx, &config);
            assert!(!out.has_errors(), "{source}: {:?}", out.diagnostics);
            let code = out.module_code();
            assert!(
                !parse_module(allocator.as_oxc(), &code, JsxLang::Tsx).has_errors(),
                "{code}"
            );
            packets.push(full_packet(source, ssr, &out));
        }
        for source in [
            "export const nodes=[1].map(n=><i>{n}</i>);",
            "export const App=()=> <Child icon={<i/>}/>;",
            "export function App(){return <p/>;}",
            "export const App=(props: Props)=><p>{props.label}</p>;",
            "export const App=({label}:{label:string})=><p>{label}</p>;",
            "export default()=> <p>{this.label}</p>;",
            "export const App=()=>{if(flag)return <i/>;return <p/>;};",
        ] {
            let allocator = Allocator::new();
            let out = compile_jsx(
                &allocator,
                source,
                JsxLang::Tsx,
                &JsxCompileConfig {
                    ssr,
                    default_mode: JsxOutputMode::Vapor,
                    ..Default::default()
                },
            );
            assert!(
                out.has_errors(),
                "unrepresented context must refuse: {source}"
            );
            assert!(out.module_code().is_empty());
            assert!(
                out.diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.end > diagnostic.start)
            );
            packets.push(full_packet(source, ssr, &out));
        }
    }
    let actual = json!(packets);
    if let Some(path) = std::env::var_os("VIZE_JSX_ADMISSION_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&actual).expect("complete packet"),
        )
        .expect("capture");
    }
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/jsx/authored-native-modules-10700/admission.json"
    ))
    .expect("frozen admission packet");
    assert_eq!(actual, expected);
}

#[test]
fn plan_ineligible_native_roots_keep_the_legacy_renderer_fallback() {
    for ssr in [false, true] {
        for source in [
            "function App(props) { return <div/>; }",
            "const App = props => <div/>;",
            "[1].map(() => <i/>);",
        ] {
            let allocator = Allocator::new();
            let out = compile_jsx(
                &allocator,
                source,
                JsxLang::Jsx,
                &JsxCompileConfig {
                    ssr,
                    default_mode: JsxOutputMode::Vapor,
                    ..Default::default()
                },
            );
            assert!(!out.has_errors(), "{source}: {:?}", out.diagnostics);
            assert_eq!(out.components.len(), 1, "{source}");
            let renderer = out.components[0].code();
            assert!(!renderer.is_empty(), "{source}");
            assert_eq!(out.module_code(), renderer, "{source}");
        }
    }
}

#[derive(Default)]
struct DirectiveBindings(Vec<(String, Option<String>)>);

impl<'a> Visit<'a> for DirectiveBindings {
    fn visit_variable_declarator(&mut self, declaration: &VariableDeclarator<'a>) {
        if declaration
            .id
            .get_identifier_name()
            .is_some_and(|name| name == "_directive_focus")
        {
            let value = declaration.init.as_ref().and_then(|expression| {
                match expression.get_inner_expression() {
                    Expression::Identifier(identifier) => Some(identifier.name.to_string()),
                    _ => None,
                }
            });
            self.0.push(("_directive_focus".to_owned(), value));
        }
        walk::walk_variable_declarator(self, declaration);
    }
}

#[test]
fn authored_vapor_directives_keep_script_setup_bindings_bare() {
    let allocator = Allocator::new();
    let source = "const vFocus = {}; export const App = () => <div v-focus/>;";
    let out = compile_jsx(
        &allocator,
        source,
        JsxLang::Jsx,
        &JsxCompileConfig {
            default_mode: JsxOutputMode::Vapor,
            ..Default::default()
        },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let module = out.module_code();
    let parsed = parse_module(allocator.as_oxc(), &module, JsxLang::Jsx);
    assert!(!parsed.has_errors(), "{module}");
    let mut bindings = DirectiveBindings::default();
    bindings.visit_program(&parsed.program);
    assert_eq!(
        bindings.0,
        [("_directive_focus".to_owned(), Some("vFocus".to_owned()))],
        "{module}"
    );
}

#[test]
fn ssr_keeps_existing_object_and_functional_roles_distinct() {
    for source in [
        "export const App=()=> <p/>;",
        "export default (props:{label:string})=> <p>{props.label}</p>;",
    ] {
        let allocator = Allocator::new();
        let out = compile_jsx(
            &allocator,
            source,
            JsxLang::Tsx,
            &JsxCompileConfig {
                ssr: true,
                ..Default::default()
            },
        );
        assert!(
            out.has_errors(),
            "unrepresented functional fallthrough: {source}"
        );
        assert!(out.module_code().is_empty());
    }
    let allocator = Allocator::new();
    let out = compile_jsx(
        &allocator,
        "export const App=(props:{label:string})=> {return <p>{props.label}</p>;};",
        JsxLang::Tsx,
        &JsxCompileConfig {
            ssr: true,
            ..Default::default()
        },
    );
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let actual = full_packet(
        "export const App=(props:{label:string})=> {return <p>{props.label}</p>;};",
        true,
        &out,
    );
    if let Some(path) = std::env::var_os("VIZE_JSX_STATEFUL_SSR_CAPTURE") {
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&actual).expect("complete packet"),
        )
        .expect("capture");
    }
    let expected:serde_json::Value=serde_json::from_str(include_str!("../../../tests/_fixtures/differential/jsx/authored-native-modules-10700/stateful-vdom-ssr.json")).expect("frozen stateful packet");
    assert_eq!(actual, expected);
}

#[test]
fn legacy_vapor_server_option_uses_the_same_genuine_ssr_component_role() {
    for source in [
        "export default()=> <p>Hello</p>;",
        "import Child from './Child.js'; export const App=()=> <Child/>;",
        "import {ref} from 'vue'; export const App=(props:{label:string})=>{const count=ref(0);return <p>{props.label}:{count.value}</p>;};",
    ] {
        let allocator = Allocator::new();
        let mut global = JsxCompileConfig {
            ssr: true,
            default_mode: JsxOutputMode::Vapor,
            ..Default::default()
        };
        global.vdom.source_map = true;
        let expected = compile_jsx(&allocator, source, JsxLang::Tsx, &global);
        assert!(!expected.has_errors(), "{:?}", expected.diagnostics);
        let mut legacy = global.clone();
        legacy.ssr = false;
        legacy.vapor.ssr = true;
        let actual = compile_jsx(&allocator, source, JsxLang::Tsx, &legacy);
        assert!(!actual.has_errors(), "{:?}", actual.diagnostics);
        assert_eq!(actual.module_code(), expected.module_code());
        assert_eq!(actual.source_map(), expected.source_map());
        assert!(
            actual
                .components
                .iter()
                .all(|component| matches!(component, vize_atelier_jsx::JsxComponent::Ssr(_)))
        );
    }
}

fn full_packet(
    source: &str,
    ssr: bool,
    output: &vize_atelier_jsx::JsxCompileOutput,
) -> serde_json::Value {
    json!({"source":source,"ssr":ssr,"module":output.module_code().as_str(),"moduleMap":output.source_map().map(|map|serde_json::from_str::<serde_json::Value>(map).expect("valid map")),"components":output.components.iter().map(|component| json!({"name":component.component_name(),"mode":component.mode().as_config_str(),"code":component.code(),"preamble":component.preamble(),"map":component.map().map(|map|serde_json::from_str::<serde_json::Value>(map).expect("valid component map")),"setup":format!("{:?}",component.component_setup()),"style":format!("{:?}",component.scoped_style())})).collect::<Vec<_>>(),"diagnostics":output.diagnostics.iter().map(|diagnostic|json!({"error":diagnostic.is_error(),"message":diagnostic.message.as_str(),"start":diagnostic.start,"end":diagnostic.end})).collect::<Vec<_>>()})
}
