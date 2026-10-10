//! #8142: decoded legacy slot emission, with selected/coordinate gaps explicit.
#![cfg(feature = "legacy-dom-differential")]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "complete authored compiler/source observations and independent anchors"
)]

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use vize_atelier_core::options::CodegenMode;
use vize_atelier_dom::{
    DomCompilerOptions, compile_template_legacy_with_options, compile_template_with_options,
};
use vize_croquis::{
    ScopeKind,
    drawer::{Drawer, DrawerOptions},
};
use vize_l0::Allocator;

const CONTROLS: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/slot-parameter-entities-8142/controls.json"
);

// One test serializes all parser/drawer/compiler observations in this binary.
#[test]
fn decoded_legacy_slot_entity_output_preserves_literal_controls_and_known_gaps() {
    let corpus: Value = serde_json::from_str(CONTROLS).expect("whole preserved sources/goldens");
    let cases = corpus["cases"].as_array().expect("authored cases");
    assert_eq!(cases.len(), 2);
    let mut observations = Vec::new();
    for case in cases {
        let source = case["source"].as_str().expect("whole original source");
        let allocator = Allocator::new();
        let (root, parser_errors) = vize_armature::parse(&allocator, source);
        let mut drawer = Drawer::with_options(DrawerOptions::full());
        drawer.draw_template(&root);
        let croquis = drawer.finish();
        let declarations = croquis
            .scopes
            .iter()
            .filter(|scope| scope.kind == ScopeKind::VSlot)
            .flat_map(|scope| {
                scope.bindings().map(|(name, binding)| {
                    json!({
                        "name": name, "declarationOffset": binding.declaration_offset,
                        "completeBindingDebug": format!("{binding:?}")
                    })
                })
            })
            .collect::<Vec<_>>();
        let mut compilers = Vec::new();
        for prefix in [false, true] {
            for legacy in [false, true] {
                let allocator = Allocator::new();
                let options = DomCompilerOptions {
                    prefix_identifiers: prefix,
                    mode: if prefix {
                        CodegenMode::Module
                    } else {
                        CodegenMode::Function
                    },
                    ..Default::default()
                };
                let (ast, errors, codegen) = if legacy {
                    compile_template_legacy_with_options(&allocator, source, options)
                } else {
                    compile_template_with_options(&allocator, source, options)
                };
                compilers.push(json!({
                    "lane": if legacy { "legacy" } else { "selected" },
                    "prefixIdentifiers": prefix,
                    "codegen": { "preamble": codegen.preamble, "code": codegen.code, "map": codegen.map },
                    "errors": errors.iter().map(|error| json!({
                        "code": format!("{:?}", error.code), "message": error.message, "loc": error.loc
                    })).collect::<Vec<_>>(),
                    "wholeAstDebug": format!("{ast:?}"),
                }));
            }
        }
        observations.push(json!({ "id": case["id"], "source": source,
            "parserErrors": parser_errors.iter().map(|error| json!({
                "code": format!("{:?}", error.code), "message": error.message, "loc": error.loc
            })).collect::<Vec<_>>(),
            "declarations": declarations, "wholeCroquisDebug": format!("{croquis:?}"),
            "wholeRootDebug": format!("{root:?}"), "compilers": compilers }));
    }
    let executable = std::env::current_exe().expect("actual compiled producer");
    let executable_bytes = std::fs::read(&executable).expect("authentic producer bytes");
    let executable_sha256 = Sha256::digest(executable_bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut output = json!({ "schema": "vize.slot-parameter-entities.actual-whole-compiler-packets", "version": 1,
        "producer": { "executable": executable, "sha256": executable_sha256, "pid": std::process::id(),
            "githubSha": std::env::var("GITHUB_SHA").ok(), "argv": std::env::args().collect::<Vec<_>>() },
        "observations": observations });
    let target = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/slot-parameter-entities-8142");
    output["runtimeReceipt"] = json!(target.join("runtime-packets.json"));
    std::fs::create_dir_all(&target).expect("whole packet retention directory");
    std::fs::write(
        target.join("compiler-packets.json"),
        serde_json::to_vec_pretty(&output).expect("whole packets"),
    )
    .expect("retain all actual compiler packets before assertions");
    run_pinned_runtime(&output, &target);

    for (case, actual) in cases
        .iter()
        .zip(output["observations"].as_array().expect("actual cases"))
    {
        assert_eq!(actual["parserErrors"], json!([]));
        let source = case["source"].as_str().expect("authored source");
        let physical = case["independentlyAuthoredPhysicalDeclarationOffset"]
            .as_u64()
            .expect("physical anchor") as usize;
        assert_eq!(source.get(physical..physical + 4), Some("item"));
        let declarations = actual["declarations"]
            .as_array()
            .expect("actual declarations");
        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0]["name"], "item");
        // This is a recorded preexisting gap, not evidence of correct mapping.
        assert_eq!(
            declarations[0]["declarationOffset"],
            case["knownDeclarationOffset"]
        );
        for (expected, observed) in case["expected"]
            .as_array()
            .expect("four full packets")
            .iter()
            .zip(actual["compilers"].as_array().expect("actual full packets"))
        {
            assert_eq!(observed["lane"], expected["lane"]);
            assert_eq!(observed["prefixIdentifiers"], expected["prefixIdentifiers"]);
            assert_eq!(
                observed["codegen"], expected["codegen"],
                "{} complete code/preamble/map",
                case["id"]
            );
            assert_eq!(
                observed["errors"], expected["errors"],
                "{} complete errors",
                case["id"]
            );
        }
    }
}

fn run_pinned_runtime(input: &Value, target: &std::path::Path) {
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/slot-parameter-entity-runtime.mjs");
    let input_bytes = serde_json::to_vec(input).expect("complete current compiler packets");
    let mut child = Command::new("node")
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("actual checksum-pinned full Vue runtime");
    let pid = child.id();
    child
        .stdin
        .take()
        .expect("runtime stdin")
        .write_all(&input_bytes)
        .expect("whole original sources and current compiler results");
    let result = child.wait_with_output().expect("actual Node runtime exit");
    std::fs::write(target.join("runtime.stdout.json"), &result.stdout)
        .expect("retain whole raw runtime stdout before assertions");
    std::fs::write(target.join("runtime.stderr.txt"), &result.stderr)
        .expect("retain whole raw runtime stderr before assertions");
    std::fs::write(
        target.join("runtime-child-process.json"),
        serde_json::to_vec_pretty(&json!({
            "pid": pid, "status": format!("{}", result.status), "code": result.status.code(),
            "inputSha256": Sha256::digest(&input_bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        }))
        .expect("actual child process receipt"),
    )
    .expect("retain actual runtime process status before assertions");
    assert!(
        result.status.success(),
        "actual runtime {pid} status={}\nstdout={}\nstderr={}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let runtime: Value =
        serde_json::from_slice(&result.stdout).expect("complete actual runtime receipt");
    assert_eq!(runtime["node"]["pid"], pid);
    assert_eq!(runtime["input"]["packets"], *input);
}
