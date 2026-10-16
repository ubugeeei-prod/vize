//! #9180: a child reload must retain the Vapor parent's setup bindings.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "runtime regression compares complete mounted packets"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcScriptOutputMode, TemplateCompileOptions, compile_sfc_for_adapter,
    parse_sfc,
};

const PARENT: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/vapor-hmr-9180/Parent.vue.txt");
const CHILD: &str =
    include_str!("../../../tests/_fixtures/differential/compiler/vapor-hmr-9180/Child.vue.txt");

fn compile(source: &str, production: bool) -> String {
    let descriptor = parse_sfc(source, Default::default()).expect("whole SFC parses");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            vapor: true,
            template: TemplateCompileOptions {
                is_prod: production,
                ..Default::default()
            },
            ..Default::default()
        },
        Default::default(),
        Default::default(),
        Default::default(),
        if production {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    )
    .expect("whole SFC compiles");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.to_string()
}

fn trace(vapor: bool, production: bool, keep_alive: bool) -> Value {
    let parent = if keep_alive {
        PARENT.replace(
            "<Child :label=\"label\" />",
            "<KeepAlive><component :is=\"Child\" :label=\"label\" /></KeepAlive>",
        )
    } else {
        PARENT.to_owned()
    };
    let module = |source: &str| {
        if vapor {
            compile(source, production)
        } else {
            source.to_owned()
        }
    };
    let input = json!({
        "vapor": vapor,
        "production": production,
        "expected": serde_json::from_str::<Value>(include_str!(
            "../../../tests/_fixtures/differential/compiler/vapor-hmr-9180/expected.json"
        )).expect("authored runtime expectations"),
        "parent": module(&parent),
        "children": (["first", "second", "third"].map(|revision|
            module(&CHILD.replace("'first'", &format!("'{revision}'")))
        )),
    });
    let runner = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/_fixtures/differential/compiler/vapor-hmr-9180/runtime.ts");
    let mut child = Command::new("node")
        .arg(runner)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("supported Vue runtime starts");
    child
        .stdin
        .take()
        .expect("runtime stdin")
        .write_all(&serde_json::to_vec(&input).expect("complete input"))
        .expect("whole compiled SFCs reach runtime");
    let output = child.wait_with_output().expect("runtime exits");
    assert!(
        output.status.success(),
        "vapor={vapor}, production={production}, keep_alive={keep_alive}\nstdout={}\nstderr={}\ninput={input}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    serde_json::from_slice(&output.stdout).expect("complete runtime packet")
}

#[test]
fn child_reloads_preserve_parent_setup_state_on_supported_vue() {
    for keep_alive in [false, true] {
        let dom = trace(false, false, keep_alive);
        assert_eq!(trace(true, false, keep_alive), dom);
    }
}

#[test]
fn production_parent_child_reactivity_and_unmount_match_vdom() {
    for keep_alive in [false, true] {
        let dom = trace(false, true, keep_alive);
        assert_eq!(trace(true, true, keep_alive), dom);
    }
}
