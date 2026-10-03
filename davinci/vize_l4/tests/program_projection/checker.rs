use super::support;
use vize_l0::Allocator;
use vize_l1::embed::Lang;
use vize_l4::targets::ts::project_program;

#[test]
fn real_typescript_checker_diagnostics_match_original_source_and_exact_mapping() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/program-checker.json")).unwrap();
    let mut projected = Vec::new();
    for case in pack["cases"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = case["source"].as_str().unwrap();
        let lang = if case["kind"] == "js" {
            Lang::Js
        } else {
            Lang::Ts
        };
        let file = support::file(&arena, source, lang).unwrap();
        assert!(file.is_complete(), "{}: {:?}", case["id"], file.issues());
        let projection = project_program(&file).unwrap();
        projected.push(serde_json::json!({"case":case,"extension":projection.source_kind().extension(),"projected":projection.document().as_str()}));
    }
    let mut child = Command::new("node")
        .args([
            "--input-type=module",
            "-e",
            include_str!("../../../../tests/tooling/support/native-program-typescript.ts"),
        ])
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&projected).unwrap())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(
        result.status.success(),
        "{}",
        std::str::from_utf8(&result.stderr).unwrap_or("non-UTF8 stderr")
    );
    let evidence: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(evidence["version"], "6.0.3");
    assert_eq!(
        evidence["cases"].as_array().unwrap().len(),
        pack["cases"].as_array().unwrap().len()
    );
    for (case, result) in pack["cases"]
        .as_array()
        .unwrap()
        .iter()
        .zip(evidence["cases"].as_array().unwrap())
    {
        let arena = Allocator::default();
        let source = case["source"].as_str().unwrap();
        let file = support::file(
            &arena,
            source,
            if case["kind"] == "js" {
                Lang::Js
            } else {
                Lang::Ts
            },
        )
        .unwrap();
        let projection = project_program(&file).unwrap();
        for diagnostic in result["diagnostics"].as_array().unwrap() {
            let start = u32::try_from(diagnostic["start"].as_u64().unwrap()).unwrap();
            let length = u32::try_from(diagnostic["length"].as_u64().unwrap()).unwrap();
            let authored = projection.map_utf16(start, length).unwrap();
            assert_eq!(
                source.get(authored.start as usize..authored.end as usize),
                diagnostic["authored"].as_str()
            );
        }
    }
    if let Ok(path) = std::env::var("VIZE_L4_PROGRAM_CHECK_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
    }
}
