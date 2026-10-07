//! Authored n8n-shaped default loops: actual Vue runtime, calls, tree and identity.
#![cfg(feature = "legacy-dom-differential")]
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "whole modules and independently authored mounted runtime oracles"
)]

use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_legacy_with_options};
use vize_l0::{Allocator, pass::BudgetObserver};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-default-slot-loop/template.vue.txt"
);
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-default-slot-loop/expected.json"
);

fn current_module() -> String {
    let allocator = Allocator::new();
    let (tree, errors) = vize_l1::parse(&allocator, SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut lowered =
        vize_l1_to_l2::lower_with_caps(&allocator, &tree, &errors, vize_l1_to_l2::LegacyCaps::VUE3);
    assert!(lowered.diagnostics.is_empty(), "{:?}", lowered.diagnostics);
    let facts = vize_l1_to_l2::pass::run_transform(&mut lowered, &mut BudgetObserver::new());
    vize_l1_to_l2::emit_dom(&lowered, &facts)
        .expect("complete native module")
        .assembled()
        .to_string()
}

#[test]
fn default_slot_loops_run_once_and_preserve_mounted_keyed_rows() {
    let allocator = Allocator::new();
    let (_, errors, legacy) =
        compile_template_legacy_with_options(&allocator, SOURCE, DomCompilerOptions::default());
    assert!(errors.is_empty(), "{errors:?}");
    let legacy = format!("{}\n{}", legacy.preamble, legacy.code);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let receipt = root.join("target/test-receipts/n8n-default-slot-loop-runtime.json");
    let input = json!({"source": SOURCE, "modules": {"legacy": legacy, "current": current_module()},
        "receipt": receipt});
    let mut child = Command::new("node")
        .arg(root.join("tests/tooling/support/n8n-default-slot-loop-runtime.mjs"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("checksum-pinned actual Vue custom renderer");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(&serde_json::to_vec(&input).expect("full input"))
        .expect("whole original source and both modules");
    let output = child.wait_with_output().expect("runtime exits");
    assert!(
        output.status.success(),
        "status={}\nstdout={}\nstderr={}\ninput={input}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let captured: Value =
        serde_json::from_slice(&output.stdout).expect("complete module/runtime receipt");
    let expected: Value = serde_json::from_str(EXPECTED).expect("independent phase/call oracle");
    for compiler in ["official", "legacy", "current"] {
        assert_eq!(
            captured["traces"][compiler], expected,
            "{compiler} has one evaluation, exact tree and stable rows in every phase"
        );
    }
}
