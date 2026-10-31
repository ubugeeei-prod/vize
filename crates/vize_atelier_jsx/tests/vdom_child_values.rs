//! Original #6890 VDOM values, independent Babel traces and untouched template controls.
#![expect(
    clippy::expect_used,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    deprecated,
    reason = "hosted integration oracle preserves complete before/head packets"
)]
use oxc_sourcemap::SourceMap;
use serde_json::{Value, json};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use vize_atelier_jsx::{JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx, parse_module};
use vize_l0::Allocator;

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/jsx-child-values-6890/original.json"
);

fn compile(source: &str, mode: JsxOutputMode, mapped: bool) -> Value {
    let allocator = Allocator::new();
    let mut options = JsxCompileConfig {
        default_mode: mode,
        ..Default::default()
    };
    options.vdom.source_map = mapped;
    let output = compile_jsx(&allocator, source, JsxLang::Jsx, &options);
    let code = output.module_code();
    assert!(
        !parse_module(allocator.as_oxc(), &code, JsxLang::Jsx).has_errors(),
        "{code}"
    );
    json!({
        "source":source, "mapped":mapped, "code":code, "functionName":"App",
        "map":output.source_map(),
        "diagnostics":output.diagnostics.iter().map(|d| json!({"message":d.message,"start":d.start,"end":d.end,"error":d.is_error()})).collect::<Vec<_>>(),
        "components":output.components.iter().map(|c| json!({"name":c.component_name(),"mode":format!("{:?}",c.mode()),"code":c.code(),"preamble":c.preamble(),"map":c.map()})).collect::<Vec<_>>()
    })
}

fn position(source: &str, offset: usize) -> (u32, u32) {
    let before = source.get(..offset).expect("UTF-8 boundary");
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

fn run_oracle(root: &Path, input: &Value, capture: Option<&Path>) -> Value {
    let mut command = Command::new("node");
    command
        .arg(root.join("tests/tooling/jsx-vapor-oracle/child-values.ts"))
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(path) = capture {
        command.env("VIZE_JSX_CHILD_TRACE_CAPTURE", path);
    }
    let mut child = command.spawn().expect("installed pinned Node oracle");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&serde_json::to_vec(input).expect("input JSON"))
        .expect("send actual modules");
    let output = child.wait_with_output().expect("oracle completion");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("complete runtime receipt")
}

fn template(source: &str) -> Value {
    use vize_atelier_core::{CodegenOptions, TransformOptions, generate, parse, transform};
    let allocator = Allocator::new();
    let (mut root, parse_errors) = parse(&allocator, source);
    assert!(parse_errors.is_empty(), "{parse_errors:?}");
    let transform_errors = transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: true,
            ..Default::default()
        },
        None,
    );
    assert!(transform_errors.is_empty(), "{transform_errors:?}");
    let output = generate(
        &root,
        CodegenOptions {
            prefix_identifiers: true,
            source_map: true,
            ..Default::default()
        },
    );
    json!({"source":source,"code":output.code,"preamble":output.preamble,"map":output.map,"diagnostics":[]})
}

#[test]
fn original_ten_values_and_maps_preserve_real_vdom_child_semantics() {
    let original: Value = serde_json::from_str(ORIGINAL).expect("unchanged original ten phases");
    let source = original["source"].as_str().expect("original source");
    let shadowed = format!("const Node=1;const document=1;const Array=1;const String=1;{source}");
    let vapor = compile(source, JsxOutputMode::Vapor, false);
    let shadowed_vapor = compile(&shadowed, JsxOutputMode::Vapor, false);
    let root = std::env::var_os("VIZE_JSX_VALUE_ORACLE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let directory = std::env::var_os("VIZE_JSX_VDOM_CHILD_CAPTURE").map(PathBuf::from);
    if let Some(directory) = &directory {
        std::fs::create_dir_all(directory).expect("capture directory");
    }
    let baseline = std::env::var_os("VIZE_JSX_VDOM_EXPECT_OLD").is_some();
    let mut cases = Vec::new();
    for (index, prefix) in ["", "// 日本語 雪🌸 'quoted'\r\n"].iter().enumerate() {
        let authored = format!("{prefix}{source}");
        let plain = compile(&authored, JsxOutputMode::Vdom, false);
        let mapped = compile(&authored, JsxOutputMode::Vdom, true);
        if let Some(directory) = &directory {
            std::fs::write(
                directory.join(format!("case-{index}-complete-producer.json")),
                serde_json::to_vec_pretty(&json!({"plain":plain,"mapped":mapped}))
                    .expect("whole compile packet"),
            )
            .expect("preserve before validation");
        }
        assert_eq!(plain["diagnostics"], json!([]));
        assert_eq!(mapped["diagnostics"], json!([]));
        assert_eq!(
            plain["code"], mapped["code"],
            "maps cannot change the producer"
        );
        let code = mapped["code"].as_str().expect("module");
        let map = SourceMap::from_json_string(mapped["map"].as_str().expect("map"))
            .expect("decoded source map");
        assert_eq!(
            map.get_source_contents().collect::<Vec<_>>(),
            [Some(authored.as_str())]
        );
        let tokens: Vec<_> = map
            .get_source_view_tokens()
            .map(|t| {
                (
                    (t.get_dst_line(), t.get_dst_col()),
                    (t.get_src_line(), t.get_src_col()),
                )
            })
            .collect();
        let generated = code.rfind("state.value").expect("actual authored read");
        let authored_read = authored.rfind("state.value").expect("authored expression");
        assert!(
            tokens.contains(&(
                position(code, generated),
                position(&authored, authored_read)
            )),
            "the raw expression retains its genuine authored coordinate: {tokens:?}"
        );
        for (mapped_flag, module) in [(false, plain), (true, mapped)] {
            let packet = json!({"original":original,"vapor":vapor,"vdom":module,"shadowedSource":shadowed,"shadowedVapor":shadowed_vapor});
            let label = format!(
                "case-{index}-{}",
                if mapped_flag { "mapped" } else { "plain" }
            );
            let capture = directory
                .as_ref()
                .map(|d| d.join(format!("{label}-trace.json")));
            if let Some(directory) = &directory {
                std::fs::write(
                    directory.join(format!("{label}-modules.json")),
                    serde_json::to_vec_pretty(&packet).expect("whole packet"),
                )
                .expect("preserve input before execution");
            }
            let receipt = run_oracle(&root, &packet, capture.as_deref());
            let expected = if baseline {
                json!([
                    true, true, true, true, true, false, false, false, false, true
                ])
            } else {
                json!([true, true, true, true, true, true, true, true, true, true])
            };
            assert_eq!(
                receipt["vdomMatchesBabel"], expected,
                "whole DOM/order/identity/getters/disposal; original input and reference are unchanged"
            );
            assert_eq!(
                receipt["wholeCorpusQualified"], false,
                "the independent Vapor VNode conflict remains explicit"
            );
            cases.push(json!({"label":label,"input":packet,"tokens":tokens,"receipt":receipt}));
        }
    }
    let mut controls = Vec::new();
    for (source, tail) in [
        (source, ""),
        (
            "const App = () => <section><main>{state.value}</main><aside>tail</aside></section>;",
            "tail",
        ),
        (
            "const App = () => <><main>{state.value}</main><aside>tail</aside></>;",
            "tail",
        ),
    ] {
        for mapped in [false, true] {
            let module = compile(source, JsxOutputMode::Vdom, mapped);
            assert_eq!(module["diagnostics"], json!([]));
            controls.push(json!({"source":source,"code":module["code"],"functionName":"App","tail":tail,"compile":module}));
        }
    }
    let control_input = json!({"original":original,"cases":controls});
    let temporary = std::env::temp_dir().join(format!("vize-vdom-children-{}", std::process::id()));
    let control_directory = directory.as_deref().unwrap_or(&temporary);
    std::fs::create_dir_all(control_directory).expect("control output directory");
    let control_input_path = control_directory.join("block-controls-input.json");
    let control_output_path = control_directory.join("block-controls-trace.json");
    std::fs::write(
        &control_input_path,
        serde_json::to_vec_pretty(&control_input).expect("whole source packet"),
    )
    .expect("capture before execution");
    let mut command = Command::new("node");
    command
        .arg(root.join("tests/tooling/jsx-vapor-oracle/vdom-child-values.ts"))
        .arg(&control_input_path)
        .arg(&control_output_path)
        .current_dir(&root);
    if baseline {
        command.env("VIZE_JSX_VDOM_EXPECT_OLD", "1");
    }
    let result = command
        .output()
        .expect("genuine Node block and mounted controls");
    std::fs::write(
        control_directory.join("block-controls.stdout"),
        &result.stdout,
    )
    .expect("raw stdout");
    std::fs::write(
        control_directory.join("block-controls.stderr"),
        &result.stderr,
    )
    .expect("raw stderr");
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let templates = [
        "<main>{{ state.value }}</main>",
        "<main v-text=\"state.value\"></main>",
    ]
    .map(template);
    if let Some(directory) = directory {
        std::fs::write(directory.join("complete.json"),serde_json::to_vec_pretty(&json!({"original":original,"baseline":baseline,"cases":cases,"templates":templates,"wholeCorpusQualified":false,"issue6890Complete":false})).expect("whole capture")).expect("capture");
    }
}
