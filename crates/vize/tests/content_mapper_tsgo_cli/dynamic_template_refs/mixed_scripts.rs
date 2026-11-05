//! Complete CLI packets for the authored mixed-script static-ref regression.

use super::{install_vue_package, output_text, workspace_root};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

mod protocol;

const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/mixed-script-static-ref-11811/"
);

#[test]
#[ignore = "requires the exact source CLI and native checker Actions lane"]
fn native_cli_mixed_script_static_refs_keep_destructured_reads_and_true_unused() {
    let runtime = crate::support::corsa_requirement::required_or_skip::<PathBuf>(None)
        .expect("this explicitly selected regression requires the native checker");
    let parent = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&parent).unwrap();
    let project = tempfile::Builder::new()
        .prefix("native-mixed-script-static-ref-11811-")
        .tempdir_in(parent)
        .unwrap();
    std::fs::create_dir(project.path().join("node_modules")).unwrap();
    install_vue_package(project.path());
    std::fs::create_dir(project.path().join("src")).unwrap();
    let inputs = materialize_inputs(project.path());
    let config: Value = serde_json::from_str(&inputs[0].1).unwrap();
    let original = &inputs[3].1;
    let unused = original.replace(
        "const { el } = useBoxObject();\n</script>",
        "const { el } = useBoxObject();\nconst trueUnused = 1;\n</script>",
    );
    assert_ne!(
        unused, *original,
        "the authored unused control must change the source"
    );
    let shadowed = original.replace("ref=\"el\"", "v-for=\"el in [1]\" ref=\"el\"");
    let cases = [
        ("original", original.as_str(), Vec::<&str>::new()),
        (
            "authored-unused",
            unused.as_str(),
            vec!["error:9:7 [TS6133] 'trueUnused' is declared but its value is never read."],
        ),
        (
            "template-shadow",
            shadowed.as_str(),
            vec!["error:8:9 [TS6133] 'el' is declared but its value is never read."],
        ),
        ("restored", original.as_str(), Vec::<&str>::new()),
    ];
    let args = ["check", "src", "--no-config", "--quiet", "--format", "json"];
    let files = [
        "src/SetupOnlyObject.vue",
        "src/WithScriptObject.vue",
        "src/WithScriptPlain.vue",
        "src/useBoxObject.ts",
    ];
    let mut outcomes = Vec::new();
    for (phase, source, messages) in cases {
        std::fs::write(project.path().join("src/WithScriptObject.vue"), source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("CORSA_PATH", &runtime)
            .args(args)
            .output()
            .unwrap();
        let expected = json!({
            "files":[
                {"file":files[0],"diagnostics":[]},
                {"file":files[1],"diagnostics":messages},
                {"file":files[2],"diagnostics":[]},
                {"file":files[3],"diagnostics":[]}
            ],
            "programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":config["compilerOptions"],"files":files}],
            "errorCount":messages.len(),"warningCount":0,"fileCount":4
        });
        println!(
            "native mixed script static ref CLI: {}",
            json!({
                "phase":phase,"binary":env!("CARGO_BIN_EXE_vize"),"argv":args,
                "runtime":runtime,"cwd":project.path(),"inputs":inputs,
                "activeSource":source,"status":output.status.code(),
                "stdoutBytes":output.stdout,"stderrBytes":output.stderr,"expected":expected
            })
        );
        outcomes.push((phase, i32::from(!messages.is_empty()), expected, output));
    }
    // Keep every original, negative and restored packet before the first verdict.
    for (phase, status, expected, output) in outcomes {
        assert_eq!(
            output.status.code(),
            Some(status),
            "{phase}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{phase}: {}", output_text(&output));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            expected,
            "{phase}: {}",
            output_text(&output),
        );
    }
}

fn materialize_inputs(project: &Path) -> [(&'static str, String); 5] {
    [
        ("tsconfig.json", "tsconfig.json.txt"),
        ("src/useBoxObject.ts", "useBoxObject.ts.txt"),
        ("src/SetupOnlyObject.vue", "SetupOnlyObject.vue.txt"),
        ("src/WithScriptObject.vue", "WithScriptObject.vue.txt"),
        ("src/WithScriptPlain.vue", "WithScriptPlain.vue.txt"),
    ]
    .map(|(file, fixture)| {
        let contents = std::fs::read_to_string(format!("{ROOT}{fixture}")).unwrap();
        std::fs::write(project.join(file), &contents).unwrap();
        (file, contents)
    })
}
