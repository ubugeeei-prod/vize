//! The real beta host must invoke the source CLI's content-mapper protocol.

use super::super::{TSGO_ENV, check_project, install_mapper_manifest};
use super::{install_vue_package, materialize_inputs, output_text, workspace_root};
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

#[test]
#[ignore = "requires the authenticated TypeScript beta host in the native-canon Actions lane"]
fn standard_tsgo_mixed_script_static_refs_keep_runtime_reads_and_authored_errors() {
    let host = std::env::var_os(TSGO_ENV)
        .map(PathBuf::from)
        .expect("this explicitly selected regression requires the real content-mapper host");
    assert!(
        host.is_file(),
        "the beta host must be a physical executable"
    );
    let version = Command::new(&host).arg("--version").output().unwrap();
    assert!(version.status.success(), "{}", output_text(&version));
    assert_eq!(version.stdout, b"Version 7.1.0-beta\n");
    assert!(version.stderr.is_empty());
    let capture = std::env::var_os("VIZE_MIXED_STATIC_REF_CAPTURE_DIR")
        .map(PathBuf::from)
        .expect("the source protocol regression must retain its actual packets");
    std::fs::create_dir_all(&capture).unwrap();
    let parent = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&parent).unwrap();
    let project = tempfile::Builder::new()
        .prefix("protocol-mixed-script-static-ref-11811-")
        .tempdir_in(parent)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::create_dir(project.path().join("src")).unwrap();
    let inputs = materialize_inputs(project.path());
    let original = &inputs[3].1;
    let unused = original.replace(
        "const { el } = useBoxObject();\n</script>",
        "const { el } = useBoxObject();\nconst trueUnused = 1;\n</script>",
    );
    assert_ne!(unused, *original);
    let shadowed = original.replace("ref=\"el\"", "v-for=\"el in [1]\" ref=\"el\"");
    let manifest = std::fs::read(project.path().join("node_modules/vize/package.json")).unwrap();
    let cases = [
        ("original", original.as_str(), ""),
        (
            "authored-unused",
            unused.as_str(),
            "src/WithScriptObject.vue(9,7): error TS6133: 'trueUnused' is declared but its value is never read.\n",
        ),
        (
            "template-shadow",
            shadowed.as_str(),
            "src/WithScriptObject.vue(8,9): error TS6133: 'el' is declared but its value is never read.\n",
        ),
        ("restored", original.as_str(), ""),
    ];
    let mut outcomes = Vec::new();
    for (phase, source, expected) in cases {
        std::fs::write(project.path().join("src/WithScriptObject.vue"), source).unwrap();
        let output = check_project(&host, project.path(), "tsconfig.json");
        let packet = json!({
            "phase":phase,"route":"actual-source-cli-content-mapper-protocol",
            "binary":env!("CARGO_BIN_EXE_vize"),"host":host,"hostVersionStdout":version.stdout,
            "manifestBytes":manifest,"cwd":project.path(),"inputs":inputs,
            "activeSource":source,"status":output.status.code(),
            "stdoutBytes":output.stdout,"stderrBytes":output.stderr,"expectedStdout":expected,
        });
        std::fs::write(
            capture.join(format!("protocol-{phase}.json")),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
        println!("mixed script static ref source protocol: {packet}");
        outcomes.push((phase, expected, output));
    }
    // Capture all four actual host outcomes before comparing any semantics.
    for (phase, expected, output) in outcomes {
        assert_eq!(
            output.status.code(),
            Some(if expected.is_empty() { 0 } else { 2 }),
            "{phase}: {}",
            output_text(&output),
        );
        assert_eq!(
            output.stdout,
            expected.as_bytes(),
            "{phase}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{phase}: {}", output_text(&output));
    }
}
