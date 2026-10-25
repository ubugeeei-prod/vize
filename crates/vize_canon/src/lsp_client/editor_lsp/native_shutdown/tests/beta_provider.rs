use super::super::{BETA_DARWIN_ARM64, BETA_LINUX_X64, image_digest};
use crate::{file_uri::path_to_file_uri, lsp_client::editor_lsp::EditorLspSession};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
use vize_l0::{FxHashMap, cstr};

const TEST: &str = "lsp_client::editor_lsp::native_shutdown::tests::beta_provider::exact_beta_editor_owner_keeps_whole_diagnostics_and_clean_eof_shutdown";
const PROVIDER: &str = "VIZE_TEST_NATIVE_BETA_TSGO";
const WORKER: &str = "VIZE_NATIVE_BETA_WORKER";
const CAPTURE: &str = "VIZE_NATIVE_BETA_CAPTURE_DIR";
const VALID: &str = "import { twice } from \"./support\";\nexport const value: number = 2;\nexport const doubled = twice(value);\n";
const SUPPORT: &str = "export const twice = (value: number): number => value * 2;\n";
const CONFIG: &str = r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","skipLibCheck":true,"noEmit":true},"include":["*.ts"]}"#;

// This is the real production editor owner, not an API-disabled fake client.
// Default CorsaProjectClient API/overlay compatibility is qualified separately.
#[ignore = "requires an exact physical beta image and complete source process capture"]
#[test]
fn exact_beta_editor_owner_keeps_whole_diagnostics_and_clean_eof_shutdown() {
    let native = std::env::var_os(PROVIDER).expect("exact beta provider is required");
    let native = Path::new(&native).canonicalize().unwrap();
    let digest = image_digest(fs::File::open(&native).unwrap()).unwrap();
    let digest: vize_l0::String = digest.iter().map(|byte| cstr!("{byte:02x}")).collect();
    assert!(matches!(
        digest.as_str(),
        BETA_LINUX_X64 | BETA_DARWIN_ARM64
    ));
    if let Some(receipt) = std::env::var_os(WORKER) {
        run_worker(&native, Path::new(&receipt));
        return;
    }
    let capture = std::env::var_os(CAPTURE).expect("whole source process capture is required");
    fs::create_dir_all(&capture).unwrap();
    let project = tempfile::tempdir().unwrap();
    let root = project.path().canonicalize().unwrap();
    fs::write(root.join("tsconfig.json"), CONFIG).unwrap();
    fs::write(root.join("main.ts"), VALID).unwrap();
    fs::write(root.join("support.ts"), SUPPORT).unwrap();
    let receipt = root.join("worker.json");
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--ignored", "--nocapture"])
        .current_dir(&root)
        .env(PROVIDER, &native)
        .env(WORKER, &receipt)
        .output()
        .unwrap();
    let result: Option<Value> = fs::read(&receipt)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap());
    let packet = json!({
        "schema":"vize.exact-beta-editor-owner-10741", "executable":native,
        "imageSha256":digest,"source":VALID,"support":SUPPORT,"config":CONFIG,
        "argv":["--exact",TEST,"--ignored","--nocapture"],
        "status":output.status.code(),"wholeStdout":output.stdout,"wholeStderr":output.stderr,
        "result":result,
        "defaultProjectClientApiCredit":false,
    });
    fs::write(
        Path::new(&capture).join("source-editor-owner.json"),
        serde_json::to_vec_pretty(&packet).unwrap(),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(0), "{packet}");
    assert_eq!(
        output.stderr, b"",
        "complete SDK and worker stderr: {packet}"
    );
    assert_whole_stdout(&output.stdout);
    let expected = [("valid", false), ("error", true), ("repair", false)]
        .into_iter()
        .map(|(phase, invalid)| {
            let text = if invalid {
                VALID.replace("= 2;", "= \"wrong\";")
            } else {
                VALID.into()
            };
            let items = if invalid {
                json!([{
                    "range":{"start":{"line":1,"character":13},"end":{"line":1,"character":18}},
                    "severity":1,"code":2322,"source":"ts",
                    "message":"Type 'string' is not assignable to type 'number'.",
                }])
            } else {
                json!([])
            };
            json!({"phase":phase,"source":text,"synchronize":{"Ok":null},
                "diagnostics":{"Ok":{"kind":"full","items":items}}})
        })
        .collect::<Vec<_>>();
    assert_eq!(
        result,
        Some(json!({
            "startup":{"Ok":null},"admitted":true,"phases":expected,
            "shutdown":{"Ok":null},"secondShutdown":{"Ok":null},
        })),
        "whole source owner DTO and lifecycle: {packet}"
    );
}

fn run_worker(native: &Path, receipt: &Path) {
    let root = std::env::current_dir().unwrap();
    let startup = EditorLspSession::spawn(native.to_str().unwrap(), &root, &root);
    let result = match startup {
        Err(error) => json!({"startup":{"Err":error}}),
        Ok(mut editor) => {
            let admitted = editor.shutdown_to_eof;
            let uri = path_to_file_uri(&root.join("main.ts"));
            let mut phases = Vec::new();
            for (phase, invalid) in [("valid", false), ("error", true), ("repair", false)] {
                let text = if invalid {
                    VALID.replace("= 2;", "= \"wrong\";")
                } else {
                    VALID.into()
                };
                let mut documents = FxHashMap::default();
                documents.insert(uri.clone().into(), text.as_str().into());
                let synchronize = editor.synchronize(&documents);
                let diagnostics = synchronize
                    .as_ref()
                    .map_err(Clone::clone)
                    .and_then(|()| editor.diagnostics(&uri));
                phases.push(json!({"phase":phase,"source":text,
                    "synchronize":synchronize,"diagnostics":diagnostics}));
            }
            // Always reap before inspecting any diagnostic result, and prove
            // the owner's closed latch makes a second shutdown inert.
            let shutdown = editor.shutdown();
            let second_shutdown = editor.shutdown();
            json!({"startup":{"Ok":null},"admitted":admitted,"phases":phases,
                "shutdown":shutdown,"secondShutdown":second_shutdown})
        }
    };
    fs::write(receipt, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
}

fn assert_whole_stdout(bytes: &[u8]) {
    let stdout = std::str::from_utf8(bytes).unwrap();
    let prefix = cstr!(
        "\nrunning 1 test\ntest {TEST} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
    );
    let rest = stdout.strip_prefix(prefix.as_str()).unwrap();
    let (filtered, duration) = rest.split_once(" filtered out; finished in ").unwrap();
    filtered.parse::<usize>().unwrap();
    let duration = duration
        .strip_suffix("s\n\n")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!(duration.is_finite() && duration >= 0.0);
}
