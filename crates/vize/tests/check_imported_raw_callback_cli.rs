//! Derived Misskey callback isolation; this does not resolve the historical 118 findings.
#![cfg(unix)]
use crate::support::{corsa_path, corsa_requirement};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
mod capture;

const PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/imported-mark-raw-callback/Parent.vue.txt"
));
const FORMATTER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/imported-mark-raw-callback/Formatter.vue.txt"
));
const I18N: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/imported-mark-raw-callback/i18n.ts.txt"
));
const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/imported-mark-raw-callback/tsconfig.json.txt"
));
const PACKAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/imported-mark-raw-callback/package.json.txt"
));
const WRONG: &str = "const wrong: number = i18n.ts._role._priority.low;\n";
const MISSING: &str = "const missing = i18n.ts._role._priority.missing;\n";

fn bytes(value: &Value) -> Vec<u8> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|n| u8::try_from(n.as_u64().unwrap()).unwrap())
        .collect()
}

fn expected_error(kind: &str) -> Value {
    match kind {
        "wrong" => json!({"code":2322,"line":11,"column":7,
            "message":"Type 'string' is not assignable to type 'number'."}),
        "missing" => json!({"code":2339,"line":11,"column":41,
            "message":"Property 'missing' does not exist on type 'Priority'."}),
        _ => Value::Null,
    }
}

fn assert_native(calls: &Value, project: &Path, source: &str, error: &Value) {
    let rows = calls.as_array().unwrap();
    let main = rows
        .iter()
        .filter(|row| row["request"]["kind"] == "main-cli")
        .collect::<Vec<_>>();
    assert_eq!(main.len(), 1, "{calls}");
    let row = main[0];
    assert_eq!(row["status"]["signal"], Value::Null, "{calls}");
    assert_eq!(row["status"]["error"], Value::Null, "{calls}");
    assert_eq!(bytes(&row["stderr"]), b"", "{calls}");
    let stdout = bytes(&row["stdout"]);
    if error.is_null() {
        assert_eq!(row["status"]["status"], 0, "{calls}");
        assert_eq!(stdout, b"", "{calls}");
    } else {
        assert_eq!(row["status"]["status"], 2, "{calls}");
        let raw = std::str::from_utf8(&stdout).unwrap();
        let (owner, tail) = raw.split_once('(').unwrap();
        let cwd = Path::new(row["request"]["cwd"].as_str().unwrap());
        let owner = fs::canonicalize(cwd.join(owner)).unwrap();
        assert!(
            owner.starts_with(cwd) || owner.starts_with(project),
            "{calls}"
        );
        assert_eq!(owner.file_name().unwrap(), "i18n.ts");
        assert_eq!(fs::read_to_string(owner).unwrap(), source);
        // Only the independently verified physical owner prefix is variable; no message/line is normalized.
        let suffix = format!(
            "{},{}): error TS{}: {}\n",
            error["line"],
            error["column"],
            error["code"],
            error["message"].as_str().unwrap()
        );
        assert_eq!(tail, suffix, "{calls}");
    }
}

#[test]
fn imported_mark_raw_getter_reads_keep_direct_callback_and_authored_error_controls() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let Some(runtime) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace)) else {
        return;
    };
    let runtime = fs::canonicalize(runtime).unwrap();
    let temporary = tempfile::tempdir().unwrap();
    let evidence = std::env::var_os("VIZE_IMPORTED_RAW_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| temporary.path().join("evidence"));
    fs::create_dir_all(&evidence).unwrap();
    let evidence = fs::canonicalize(evidence).unwrap();
    let providers = capture::providers(workspace, &evidence, &runtime);
    let project = temporary.path().join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::create_dir(project.join("node_modules")).unwrap();
    std::os::unix::fs::symlink(
        providers["vue"]["root"].as_str().unwrap(),
        project.join("node_modules/vue"),
    )
    .unwrap();
    for (name, body) in [
        ("package.json", PACKAGE),
        ("tsconfig.json", CONFIG),
        ("src/Parent.vue", PARENT),
        ("src/Formatter.vue", FORMATTER),
    ] {
        fs::write(project.join(name), body).unwrap();
        let saved = evidence.join("source").join(name);
        fs::create_dir_all(saved.parent().unwrap()).unwrap();
        fs::write(saved, body).unwrap();
    }
    let proxy = capture::forwarder(&evidence);
    let vue_tsc = Path::new(providers["vueTsc"]["root"].as_str().unwrap()).join("bin/vue-tsc.js");
    let phases = [
        ("original", ""),
        ("wrong", WRONG),
        ("repair-wrong", ""),
        ("restore-wrong", WRONG),
        ("missing", MISSING),
        ("repair-missing", ""),
        ("restore-missing", MISSING),
    ];
    let mut observations = Vec::new();
    for (phase, extra) in phases {
        let source = format!("{I18N}{extra}");
        fs::write(project.join("src/i18n.ts"), &source).unwrap();
        let destination = evidence.join(phase);
        fs::create_dir_all(destination.join("native-calls")).unwrap();
        let cache = temporary.path().join("cache").join(phase);
        fs::create_dir_all(&cache).unwrap();
        let cli = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&project)
            .env("CORSA_PATH", &proxy)
            .env("VIZE_RAW_NATIVE", &runtime)
            .env("VIZE_RAW_CAPTURE", destination.join("native-calls"))
            .env("XDG_CACHE_HOME", &cache)
            .args([
                "check",
                "--no-config",
                "--quiet",
                "--format",
                "json",
                "--show-virtual-ts",
                "--tsconfig",
                "tsconfig.json",
                ".",
            ])
            .output()
            .unwrap();
        capture::retain(&destination.join("vize"), &cli, &source);
        let js = Command::new("node")
            .current_dir(&project)
            .arg(&vue_tsc)
            .args([
                "--noEmit",
                "--pretty",
                "false",
                "--project",
                "tsconfig.json",
            ])
            .output()
            .unwrap();
        capture::retain(&destination.join("vue-tsc"), &js, &source);
        let calls = capture::native_calls(&destination.join("native-calls"));
        observations.push((phase, source, cli, js, calls));
    }
    // Every complete stream, source and native outcome above is retained before the semantic verdict.
    capture::save(
        &evidence.join("scope.json"),
        &json!({"derivedFixture":true,"historical118Resolved":false,
        "workspaceRole":"tests package actual Vue/TypeScript/vue-tsc distributions", "historicalRole":"Rc13/provider3.3.13 plus MisskeyVue3.5.28 is separate",
        "loadedNativeFiles":"UNOBSERVED","nativeCompilerSource":"UNJOINED","timingUsable":false,"phases":7}),
    );
    assert_eq!(providers["vue"]["version"], "3.6.0-beta.10");
    assert_eq!(providers["typescript"]["version"], "6.0.3");
    assert_eq!(providers["vueTsc"]["version"], "3.3.11");
    assert_eq!(providers["languageCore"]["version"], "3.3.11");
    let baseline: Value = serde_json::from_slice(&observations[0].2.stdout).unwrap();
    let mut keys = baseline
        .as_object()
        .unwrap()
        .keys()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "errorCount",
            "fileCount",
            "files",
            "programs",
            "warningCount"
        ]
    );
    assert_eq!(
        baseline["programs"],
        json!([{"root":".","tsconfig":"tsconfig.json","compilerOptions":{
        "strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","skipLibCheck":true,"noEmit":true,"types":[]
    },"files":["src/Formatter.vue","src/Parent.vue","src/i18n.ts"]}])
    );
    assert_eq!(baseline["errorCount"], 0, "{baseline}");
    assert_eq!(baseline["warningCount"], 0, "{baseline}");
    assert_eq!(baseline["fileCount"], 3, "{baseline}");
    assert_eq!(
        baseline["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["file"].clone())
            .collect::<Vec<_>>(),
        vec![
            json!("src/Formatter.vue"),
            json!("src/Parent.vue"),
            json!("src/i18n.ts")
        ]
    );
    for file in baseline["files"].as_array().unwrap() {
        assert_eq!(file["diagnostics"], json!([]));
    }
    for (phase, source, cli, js, calls) in &observations {
        let kind = if phase.ends_with("wrong") && phase != &"repair-wrong" {
            "wrong"
        } else if phase.ends_with("missing") && phase != &"repair-missing" {
            "missing"
        } else {
            ""
        };
        let error = expected_error(kind);
        let bad = !error.is_null();
        assert_eq!(
            cli.status.code(),
            Some(if bad { 1 } else { 0 }),
            "{phase}: {:?}",
            cli
        );
        assert_eq!(
            js.status.code(),
            Some(if bad { 2 } else { 0 }),
            "{phase}: {:?}",
            js
        );
        assert_eq!(js.stderr, b"", "{phase}: {:?}", js);
        let expected_js = if bad {
            format!(
                "src/i18n.ts({},{}): error TS{}: {}\n",
                error["line"],
                error["column"],
                error["code"],
                error["message"].as_str().unwrap()
            )
        } else {
            "".to_owned()
        };
        assert_eq!(js.stdout, expected_js.as_bytes(), "{phase}");
        let mut expected = baseline.clone();
        if bad {
            expected["errorCount"] = json!(1);
            expected["files"][2]["diagnostics"] = json!([format!(
                "error:{}:{} [TS{}] {}",
                error["line"],
                error["column"],
                error["code"],
                error["message"].as_str().unwrap()
            )]);
        }
        assert_eq!(
            serde_json::from_slice::<Value>(&cli.stdout).unwrap(),
            expected,
            "{phase}"
        );
        for call in calls.as_array().unwrap() {
            assert_eq!(call["request"]["executable"], json!(runtime));
            assert_eq!(
                call["request"]["imageBefore"],
                providers["native"]["sha256"]
            );
            assert_eq!(
                call["request"]["node"]["executable"],
                providers["node"]["executable"]
            );
            assert_eq!(
                call["request"]["node"]["version"],
                providers["node"]["version"]
            );
            let kind = call["request"]["kind"].as_str().unwrap();
            assert!(
                ["main-cli", "option-probe", "version"].contains(&kind),
                "{calls}"
            );
            if kind == "main-cli" {
                let argv = call["request"]["argv"].as_array().unwrap();
                assert_eq!(argv.len(), 9, "{calls}");
                assert_eq!(
                    &argv[..4],
                    &[
                        json!("--checkers"),
                        json!("1"),
                        json!("--incremental"),
                        json!("--tsBuildInfoFile")
                    ]
                );
                assert_eq!(
                    &argv[5..],
                    &[
                        json!("--pretty"),
                        json!("false"),
                        json!("--project"),
                        call["request"]["config"].clone()
                    ]
                );
            }
        }
        assert_native(calls, &project, source, &error);
        assert_eq!(
            capture::digest(&fs::read(&runtime).unwrap()),
            providers["native"]["sha256"].as_str().unwrap()
        );
    }
    for (left, right) in [(0, 2), (0, 5), (1, 3), (4, 6)] {
        assert_eq!(observations[left].2.stdout, observations[right].2.stdout);
        assert_eq!(observations[left].2.stderr, observations[right].2.stderr);
        assert_eq!(observations[left].3.stdout, observations[right].3.stdout);
        assert_eq!(observations[left].3.stderr, observations[right].3.stderr);
    }
}
