//! Genuine same-config native diagnostics, with complete before/after custody.
#![expect(clippy::expect_used, reason = "required native fixture authority")]

use super::{CORPUS, GeneratedVueFile, WORKSPACE, packet, project, sources};
use crate::{corsa_client::CorsaProjectClient, file_uri::path_to_file_uri};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use vize_carton::{String, cstr};

mod comment_boundaries;
mod public;
mod rows;
use rows::{Row, mapped_rows};

fn native() -> Option<PathBuf> {
    let required = std::env::var("VIZE_TEST_REQUIRE_TSGO").as_deref() == Ok("1");
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        assert!(
            !required,
            "required descriptor native laws cannot be disabled"
        );
        return None;
    }
    Some(
        vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                explicit_path: std::env::var_os("CORSA_PATH").as_deref().map(Path::new),
                project_root: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
            },
        )
        .expect("the actual native runtime is required")
        .canonicalize()
        .unwrap(),
    )
}

fn install(root: &Path, issue: &str) -> PathBuf {
    let workspace = root.join("test-workspace");
    let project = workspace
        .join("tsc")
        .join(cstr!("_failed_#{issue}").as_str());
    fs::create_dir_all(&project).unwrap();
    fs::copy(
        Path::new(WORKSPACE).join("tsconfig.base.json"),
        workspace.join("tsconfig.base.json"),
    )
    .unwrap();
    assert_eq!(
        fs::read(workspace.join("tsconfig.base.json")).unwrap(),
        fs::read(Path::new(CORPUS).join("tsconfig.base.json.txt")).unwrap()
    );
    fs::copy(
        Path::new(WORKSPACE)
            .join("tsc")
            .join(cstr!("_failed_#{issue}").as_str())
            .join("tsconfig.json"),
        project.join("tsconfig.json"),
    )
    .unwrap();
    assert_eq!(
        fs::read(project.join("tsconfig.json")).unwrap(),
        fs::read(Path::new(CORPUS).join(cstr!("{issue}-tsconfig.json.txt").as_str())).unwrap()
    );
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let vue = repo
        .join("tests/node_modules/vue-language-tools-fixture-vue")
        .canonicalize()
        .unwrap();
    let manifest: Value =
        serde_json::from_slice(&fs::read(vue.join("package.json")).unwrap()).unwrap();
    assert_eq!(manifest["version"], "3.6.0-rc.6");
    let modules = workspace.join("node_modules");
    fs::create_dir_all(&modules).unwrap();
    std::os::unix::fs::symlink(&vue, modules.join("vue")).unwrap();
    std::os::unix::fs::symlink(vue.parent().unwrap().join("@vue"), modules.join("@vue")).unwrap();
    std::os::unix::fs::symlink(
        repo.join("tests/node_modules/vue-component-type-helpers")
            .canonicalize()
            .unwrap(),
        modules.join("vue-component-type-helpers"),
    )
    .unwrap();
    std::os::unix::fs::symlink(
        repo.join("npm/builder/vite/node_modules/vite")
            .canonicalize()
            .unwrap(),
        modules.join("vite"),
    )
    .unwrap();
    project
}

fn diagnose(
    root: &Path,
    native: &Path,
    phase: &str,
    inputs: &[(&str, std::string::String, GeneratedVueFile)],
) -> (Vec<Row>, Value) {
    for (name, source, output) in inputs {
        fs::write(root.join(name), source).unwrap();
        fs::write(root.join(cstr!("{name}.ts").as_str()), output.code.as_str()).unwrap();
    }
    let config = root.join("tsconfig.json");
    let cli = Command::new(native)
        .args(["--checkers", "1", "--pretty", "false", "--project"])
        .arg(&config)
        .current_dir(root)
        .output()
        .unwrap();
    let mut client =
        CorsaProjectClient::new_for_workspace_config(Some(native.to_str().unwrap()), root, &config)
            .expect("same-config native diagnostic transport must initialize");
    let uris: Vec<String> = inputs
        .iter()
        .map(|(name, _, _)| path_to_file_uri(&root.join(cstr!("{name}.ts").as_str())))
        .collect();
    client.begin_batch_test_receipt(true);
    let raw = client
        .request_diagnostics_batch(&uris)
        .expect("native syntax and semantic groups must complete");
    let authority = client.batch_test_receipt(uris.first().unwrap());
    let mut retained = json!({
        "phase":phase,"wholeAuthoredConfig":fs::read_to_string(&config).unwrap(),
        "wholeAuthoredBaseConfig":fs::read_to_string(root.parent().unwrap().parent().unwrap().join("tsconfig.base.json")).unwrap(),
        "requestedUris":uris,"nativeAuthority":authority,"wholeRawDiagnosticVector":raw,
        "wholeCliExit":cli.status.code(),"wholeCliStdout":std::string::String::from_utf8(cli.stdout).unwrap(),
        "wholeCliStderr":std::string::String::from_utf8(cli.stderr).unwrap(),
        "wholeInputs":inputs.iter().map(|(name,source,output)| json!({"file":name,"wholeAuthoredSource":source,"wholeGeneratedPacket":packet(output)})).collect::<Vec<_>>(),
    });
    // Preserve an unexpected response even if its source mapping cannot complete.
    retain(root, phase, &retained);
    let mapped = mapped_rows(root, inputs, &raw);
    retained["wholeMappedDiagnosticVector"] = json!(mapped);
    retain(root, phase, &retained);
    assert_eq!(authority["selectedConfig"], json!(config));
    assert!(authority["project"].is_object(), "{authority}");
    assert!(authority["snapshotProjects"].is_array(), "{authority}");
    assert!(
        authority["normalizedRequestedOptions"].is_object(),
        "{authority}"
    );
    assert_eq!(
        authority["project"]["compilerOptions"], authority["normalizedRequestedOptions"],
        "{authority}"
    );
    assert!(
        matches!(
            authority["mode"].as_str(),
            Some("project-api" | "editor-lsp")
        ),
        "{authority}"
    );
    (mapped, retained)
}

fn retain(root: &Path, phase: &str, packet: &Value) {
    if let Some(directory) = std::env::var_os("VIZE_NATIVE_SYNTAX_CAPTURE_DIR") {
        let directory = PathBuf::from(directory).join("malformed-setup");
        fs::create_dir_all(&directory).unwrap();
        let case = root.file_name().unwrap().to_str().unwrap();
        fs::write(
            directory.join(cstr!("{case}-{phase}.json").as_str()),
            serde_json::to_vec_pretty(packet).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn original_malformed_setup_native_vectors_keep_all_syntax_and_repair_only_false_generics() {
    let Some(native) = native() else { return };
    let mut cases = Vec::new();
    for issue in ["5071", "5823"] {
        let temporary = tempfile::tempdir().unwrap();
        let root = install(temporary.path(), issue);
        let mut old = Vec::new();
        let mut fresh = Vec::new();
        let mut projections = Vec::new();
        for (name, source) in sources(issue) {
            let (before, after, projection) = project(&source);
            old.push((name, source.clone(), before));
            fresh.push((name, source, after));
            projections.push(json!({"file":name,"wholeProjection":projection}));
        }
        retain(&root, "descriptor-projections", &json!(projections));
        let (before, before_packet) = diagnose(&root, &native, "before", &old);
        let (after, after_packet) = diagnose(&root, &native, "after", &fresh);
        cases.push((issue, before, before_packet, after, after_packet));
    }
    for (issue, before, before_packet, after, after_packet) in cases {
        if issue == "5823" {
            let false_row = (
                "main.vue".into(),
                2,
                11,
                Some(2304),
                Some(1),
                "Cannot find name 'T'.".into(),
            );
            assert_eq!(
                before.iter().filter(|row| **row == false_row).count(),
                1,
                "{before_packet}"
            );
            let preserved: Vec<_> = before.into_iter().filter(|row| *row != false_row).collect();
            assert_eq!(after, preserved, "{after_packet}");
        } else {
            let original = vec![
                (
                    "withScript.vue".into(),
                    0,
                    18,
                    Some(1005),
                    Some(1),
                    "';' expected.".into(),
                ),
                (
                    "withoutScript.vue".into(),
                    1,
                    25,
                    Some(1005),
                    Some(1),
                    "';' expected.".into(),
                ),
            ];
            assert_eq!(before, original, "{before_packet}");
            assert_eq!(after, before, "{after_packet}");
        }
        let syntax: Vec<_> = after
            .iter()
            .filter(|row| matches!(row.3, Some(1005 | 1109)))
            .map(|row| (row.0.as_str(), row.1, row.2, row.3))
            .collect();
        assert_eq!(
            syntax,
            if issue == "5823" {
                vec![("main.vue", 5, 12, Some(1109))]
            } else {
                vec![
                    ("withScript.vue", 0, 18, Some(1005)),
                    ("withoutScript.vue", 1, 25, Some(1005)),
                ]
            }
        );
    }
}

#[test]
fn malformed_setup_keeps_real_undeclared_and_wrong_type_semantics() {
    let Some(native) = native() else { return };
    let temporary = tempfile::tempdir().unwrap();
    let root = install(temporary.path(), "5823");
    let original = sources("5823").pop().unwrap().1;
    let (prefix, suffix) = original.rsplit_once("</script>").unwrap();
    let source = std::string::String::from(cstr!("{prefix}const missing = __vizeReallyMissing;\nconst mismatch: number = \"not-a-number\";\n</script>{suffix}").as_str());
    assert_eq!(
        source,
        fs::read_to_string(Path::new(CORPUS).join("real-errors.vue.txt")).unwrap()
    );
    let (_, after, projection) = project(&source);
    retain(&root, "real-semantic-projection", &projection);
    let (rows, packet) = diagnose(
        &root,
        &native,
        "real-semantic-errors",
        &[("main.vue", source.clone(), after)],
    );
    let missing = source.find("__vizeReallyMissing").unwrap() as u32;
    let (line, column) = crate::batch::source_map::offset_to_line_col(&source, missing).unwrap();
    assert_eq!(
        rows.iter()
            .filter(|row| **row
                == (
                    "main.vue".into(),
                    line,
                    column,
                    Some(2304),
                    Some(1),
                    "Cannot find name '__vizeReallyMissing'.".into()
                ))
            .count(),
        1,
        "{packet}"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.3 == Some(2322)
                && row.5 == "Type 'string' is not assignable to type 'number'.")
            .count(),
        1,
        "{packet}"
    );
    assert_eq!(
        rows.iter().filter(|row| row.3 == Some(1109)).count(),
        1,
        "{packet}"
    );
    assert!(
        !rows
            .iter()
            .any(|row| row.3 == Some(2304) && row.5 == "Cannot find name 'T'."),
        "{packet}"
    );
}
