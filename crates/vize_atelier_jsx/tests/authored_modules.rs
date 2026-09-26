//! Execute complete emitted modules under the official pinned Vue runtime.
#![expect(
    clippy::expect_used,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "integration fixtures use process/JSON strings and assert compiler output"
)]

#[path = "authored_modules/scope.rs"]
mod scope;

use oxc_sourcemap::SourceMap;
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, compile_jsx};
use vize_l0::Allocator;

fn compile(source: &str) -> vize_l0::String {
    compile_with_config(source, &JsxCompileConfig::default())
}

fn compile_with_config(source: &str, config: &JsxCompileConfig) -> vize_l0::String {
    let arena = Allocator::new();
    let out = compile_jsx(&arena, source, JsxLang::Tsx, config);
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let module = out.module_code();
    let parsed = vize_atelier_jsx::parse_module(arena.as_oxc(), &module, JsxLang::Tsx);
    assert!(
        !parsed.has_errors(),
        "emitted module: {module}\n{:?}",
        parsed.diagnostics
    );
    module
}

#[test]
fn complete_tsx_modules_execute_with_imports_defaults_and_mixed_roots() {
    let cases = [
        (
            "expression",
            r#"
            import Child from "./Child";
            export const suffix = "!";
            export function Card(props: { label: string }) {
                const label = props.label + suffix;
                return <Child label={label + arguments[0].label.slice(0, 0)}/>;
            }
            export default <T extends string,>(props: { label: T }) => <Card label={props.label}/>;
        "#,
        ),
        (
            "mixed",
            r#"
            import { ref } from "vue";
            export const Stateful = () => {
                const count = ref(0);
                const increment = () => count.value++;
                return <button onClick={increment}>{count.value}</button>;
            };
            export const Pure = (props: {label:string}) => <i>{props.label}</i>;
            export default Stateful;
        "#,
        ),
        (
            "default",
            r#"
            export const App = ({ slot = <i data-kind="fallback"/> }: {slot?: any}) => {
                const kind = slot.type;
                return <div>{kind}</div>;
            };
            export default App;
        "#,
        ),
        (
            "typed",
            r#"
            export default function marker() { return "retained"; }
            export const Typed = (props: { label: string; amount?: number }) => {
                function readLabel(this: { prefix: string }, label: string) {
                    return this.prefix + arguments[0];
                }
                class Model { self = this; static self; static { this.self = this; } }
                const model = new Model();
                const read = () => readLabel.call({ prefix: "" }, props.label)
                    + (model.self === model && Model.self === Model ? "" : "lost-class-receiver");
                return <p>{read()}</p>;
            }; export const afterTyped = "retained-after-setup";
        "#,
        ),
        (
            "options",
            r#"
            export const widgets = { marker: "kept", render: () => { return <i>helper</i>; } };
            export default {
                data() { return { label: "lexical" }; },
                render() { return <p>{this.label}:{arguments[0].label}</p>; }
            };
        "#,
        ),
        (
            "factory",
            r#"
            import { label } from "./Plain";
            export const make = function (label: string) {
                return () => { return <p>{label}</p>; };
            };
            export default make(label);
        "#,
        ),
        (
            "slots",
            r#"
            import Child from "./Child";
            const s = "outer";
            export default (props: { label: string }) => <Child label={props.label}>{{
                default: (s: {label:string}) => <i>{s.label}</i>,
                footer: (_ctx: {label:string}) => <b>{_ctx.label}</b>,
                outer: () => <u>{s}</u>
            }}</Child>;
        "#,
        ),
        (
            "plain",
            "export interface Props { label: string }\nexport const label: string = 'retained-factory';",
        ),
    ];
    let modules: serde_json::Map<_, _> = cases
        .into_iter()
        .map(|(name, source)| {
            let config = JsxCompileConfig {
                compat: if name == "slots" {
                    JsxCompatMode::Babel
                } else {
                    JsxCompatMode::Native
                },
                ..Default::default()
            };
            (
                name.to_owned(),
                serde_json::Value::String(compile_with_config(source, &config).to_string()),
            )
        })
        .collect();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = Command::new("node")
        .arg(root.join("tests/tooling/support/tsx-authored-module-runtime.mjs"))
        .current_dir(&root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node and installed runtime dependencies are required");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&serde_json::to_vec(&modules).expect("payload"))
        .expect("write payload");
    let output = child.wait_with_output().expect("runtime process");
    assert!(
        output.status.success(),
        "{}\n{}",
        std::string::String::from_utf8_lossy(&output.stdout),
        std::string::String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::string::String::from_utf8_lossy(&output.stdout).trim(),
        "7 mounted TSX module scenarios passed"
    );
}

#[test]
fn modules_without_jsx_keep_exact_source_and_identity_maps() {
    let source = "// 😀 保全\r\nimport type { Ref } from 'vue';\r\nexport interface Props { label: Ref<string> }\r\nexport const label: string = 'retained';\r\nexport default label;\r\n";
    for default_mode in [
        vize_atelier_jsx::JsxOutputMode::Vdom,
        vize_atelier_jsx::JsxOutputMode::Vapor,
    ] {
        let allocator = Allocator::new();
        let mut config = JsxCompileConfig {
            default_mode,
            ..Default::default()
        };
        config.vdom.source_map = true;
        let output = compile_jsx(&allocator, source, JsxLang::Tsx, &config);
        assert_eq!(output.diagnostics.len(), 0);
        assert_eq!(output.components.len(), 0);
        assert_eq!(output.module_code().as_str(), source);
        let map = SourceMap::from_json_string(output.source_map().expect("identity map"))
            .expect("valid identity map");
        assert_eq!(
            map.get_source_contents().collect::<Vec<_>>(),
            [Some(source)]
        );
        assert!(map.get_source_view_tokens().all(|token| {
            token.get_dst_line() == token.get_src_line()
                && token.get_dst_col() == token.get_src_col()
        }));
        let original = position(source, source.find("export const label").expect("export"));
        let token = map
            .lookup_source_view_token(&map.generate_lookup_table(), original.0, original.1)
            .expect("export mapping");
        assert_eq!((token.get_src_line(), token.get_src_col()), original);
    }
}

#[test]
fn composed_module_maps_keep_full_unicode_source_and_authored_coordinates() {
    let source = "// 😀 日本語\r\nexport const marker = '保全';\r\nexport default (props: {label:string}) => <p title=\"😀\">{props.label}</p>;\r\nexport const Other = () => <i>{marker}</i>;\r\n";
    let arena = Allocator::new();
    let mut config = JsxCompileConfig::default();
    config.vdom.source_map = true;
    let out = compile_jsx(&arena, source, JsxLang::Tsx, &config);
    assert!(!out.has_errors(), "{:?}", out.diagnostics);
    let module = out.module_code();
    let map = SourceMap::from_json_string(out.source_map().expect("complete module map"))
        .expect("valid map");
    assert_eq!(
        map.get_source_contents().collect::<Vec<_>>(),
        [Some(source)]
    );
    let retained = module.find("export const marker").expect("retained export");
    let (line, col) = position(&module, retained);
    assert!(
        map.get_source_view_tokens()
            .any(|token| token.get_dst_line() == line
                && token.get_dst_col() == col
                && token.get_src_line() == 1
                && token.get_src_col() == 0)
    );
    let expression = source.lines().nth(2).expect("line");
    let original_column = expression
        .get(..expression.find("props.label").expect("authored expression"))
        .expect("boundary")
        .encode_utf16()
        .count() as u32;
    let (generated_line, generated_column) = position(
        &module,
        module.find("props.label").expect("generated expression"),
    );
    assert!(map.get_source_view_tokens().any(|token| {
        token.get_dst_line() == generated_line
            && token.get_dst_col() == generated_column
            && token.get_src_line() == 2
            && token.get_src_col() == original_column
    }));
    assert!(
        map.get_source_view_tokens()
            .all(|token| token.get_src_line() < 5)
    );
}

fn position(source: &str, offset: usize) -> (u32, u32) {
    let prefix = source.get(..offset).expect("boundary");
    let line = prefix.bytes().filter(|&b| b == b'\n').count() as u32;
    let column = prefix
        .rsplit('\n')
        .next()
        .expect("last line")
        .encode_utf16()
        .count() as u32;
    (line, column)
}
