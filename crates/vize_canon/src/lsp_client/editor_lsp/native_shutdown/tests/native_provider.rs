use crate::{corsa_client::CorsaProjectClient, file_uri::path_to_file_uri};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
use vize_l0::cstr;

const TEST: &str = "lsp_client::editor_lsp::native_shutdown::tests::native_provider::pinned_mapper_sessions_keep_complete_diagnostics_and_clean_owned_shutdown";
const PROVIDER: &str = "VIZE_TEST_NATIVE_MAPPER_TSGO";
const WORKER: &str = "VIZE_NATIVE_PROVIDER_WORKER";
const CAPTURE: &str = "VIZE_NATIVE_PROVIDER_CAPTURE_DIR";
const VALID: &str = "function probe<T extends string>() {\nconst independent: number = 1;\nvoid independent;\nconst genericValue: T | undefined = undefined;\nvoid genericValue;\n}\n";
const CONFIG: &str = r#"{"compilerOptions":{"jsx":"preserve","module":"ESNext","moduleResolution":"Bundler","noEmit":true,"skipLibCheck":true,"strict":true,"target":"ESNext"},"files":["probe.ts"]}"#;

// This is an explicit ignored opt-in, never a successful early return when the
// independently built provider is absent. The Actions selector runs --ignored.
#[ignore = "requires the physical pinned mapper provider and complete capture directory"]
#[test]
fn pinned_mapper_sessions_keep_complete_diagnostics_and_clean_owned_shutdown() {
    let native = std::env::var_os(PROVIDER).expect("the actual pinned mapper provider is required");
    let native = Path::new(&native).canonicalize().unwrap();
    let digest = super::super::image_digest(fs::File::open(&native).unwrap()).unwrap();
    let digest: vize_l0::String = digest.iter().map(|byte| cstr!("{byte:02x}")).collect();
    assert_eq!(digest.as_str(), super::super::MAPPER_LINUX_X64);
    if let Some(receipt) = std::env::var_os(WORKER) {
        run_worker(&native, Path::new(&receipt));
        return;
    }
    let capture = std::env::var_os(CAPTURE).expect("whole-stream custody is required");
    fs::create_dir_all(&capture).unwrap();
    let project = tempfile::tempdir().unwrap();
    let root = project.path().canonicalize().unwrap();
    fs::write(root.join("tsconfig.json"), CONFIG).unwrap();
    for (phase, invalid) in [("valid", false), ("error", true), ("repaired", false)] {
        let text = if invalid {
            VALID.replace("= 1;", "= 'bad';")
        } else {
            VALID.into()
        };
        fs::write(root.join("probe.ts"), &text).unwrap();
        let receipt = root.join("worker.json");
        let _ = fs::remove_file(&receipt);
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
            "schema":"vize.pinned-provider-session-10599", "phase":phase,
            "executable":native,"imageSha256":digest,"source":text,"config":CONFIG,
            "argv":["--exact",TEST,"--ignored","--nocapture"],
            "status":output.status.code(),"wholeStdout":output.stdout,"wholeStderr":output.stderr,
            "result":result,
        });
        fs::write(
            Path::new(&capture).join(cstr!("{phase}.json").as_str()),
            serde_json::to_vec_pretty(&packet).unwrap(),
        )
        .unwrap();
        assert_eq!(output.status.code(), Some(0), "{packet}");
        assert!(
            output.stderr.is_empty(),
            "complete provider stderr: {packet}"
        );
        assert_whole_test_stdout(&output.stdout);
        let expected = if invalid {
            json!([{
                "code":2322,"message":"Type 'string' is not assignable to type 'number'.",
                "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},
                "severity":1,"source":"ts",
            }])
        } else {
            json!([])
        };
        assert_eq!(
            result,
            Some(json!({
                "source":text,"config":CONFIG,"startup":{"Ok":null},
                "diagnostics":{"Ok":expected},"admitted":true,"shutdown":{"Ok":null},
            })),
            "whole diagnostic DTO and owned lifecycle: {packet}"
        );
    }
}

fn run_worker(native: &Path, receipt: &Path) {
    let root = std::env::current_dir().unwrap();
    let source = fs::read_to_string(root.join("probe.ts")).unwrap();
    let config = fs::read_to_string(root.join("tsconfig.json")).unwrap();
    let mut client = CorsaProjectClient::new_for_workspace(native.to_str(), &root).unwrap();
    let diagnostics = client.request_diagnostics(&path_to_file_uri(&root.join("probe.ts")));
    let admitted = client
        .editor_lsp
        .as_ref()
        .is_some_and(|editor| editor.shutdown_to_eof);
    // Always close before checking or serializing any diagnostic outcome.
    let shutdown = client.shutdown();
    fs::write(
        receipt,
        serde_json::to_vec_pretty(&json!({
            "source":source,"config":config,"startup":{"Ok":null},
            "diagnostics":diagnostics,"admitted":admitted,"shutdown":shutdown,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn assert_whole_test_stdout(bytes: &[u8]) {
    let stdout = std::str::from_utf8(bytes).unwrap();
    let prefix = cstr!(
        "\nrunning 1 test\ntest {TEST} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; "
    );
    let rest = stdout
        .strip_prefix(prefix.as_str())
        .unwrap_or_else(|| panic!("unexpected whole stdout: {stdout:?}"));
    let (filtered, duration) = rest.split_once(" filtered out; finished in ").unwrap();
    filtered.parse::<usize>().unwrap();
    let duration = duration
        .strip_suffix("s\n\n")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!(duration.is_finite() && duration >= 0.0);
}
