//! Derived callback isolation on the independently observed Linux7.0.2 workspace role.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use crate::support::{corsa_path, corsa_requirement};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
mod capture;
mod oracles;

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

const NATIVE_IMAGE: &str = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";

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
        ("src/i18n.ts", I18N),
    ] {
        fs::write(project.join(name), body).unwrap();
        let saved = evidence.join("source").join(name);
        fs::create_dir_all(saved.parent().unwrap()).unwrap();
        fs::write(saved, body).unwrap();
    }
    let proxy = capture::forwarder(&evidence);
    let cache = temporary.path().join("cache");
    fs::create_dir_all(&cache).unwrap();
    let vue_tsc = Path::new(providers["vueTsc"]["root"].as_str().unwrap()).join("bin/vue-tsc.js");
    let phases = [
        ("original", ""),
        ("wrong", "wrong"),
        ("repair-wrong", ""),
        ("restore-wrong", "wrong"),
        ("missing", "missing"),
        ("repair-missing", ""),
        ("restore-missing", "missing"),
    ];
    let mut observations = Vec::new();
    for (phase, kind) in phases {
        let source = oracles::parent(kind);
        fs::write(project.join("src/Parent.vue"), source.as_bytes()).unwrap();
        let destination = evidence.join(phase);
        fs::create_dir_all(destination.join("native-calls")).unwrap();
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
        capture::retain(&destination.join("vize"), &cli, source.as_str());
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
        capture::retain(&destination.join("vue-tsc"), &js, source.as_str());
        let calls = capture::native_calls(&destination.join("native-calls"));
        observations.push((phase, kind, source, cli, js, calls));
    }
    let geometry_inputs = json!(
        observations
            .iter()
            .map(|(phase, _, _, cli, _, _)| {
                let packet: Value = serde_json::from_slice(&cli.stdout).unwrap();
                json!({"phase":phase,"virtualTs":packet["files"][1]["virtualTs"]})
            })
            .collect::<Vec<_>>()
    );
    let geometry = capture::geometry(&evidence, &providers, &geometry_inputs);
    capture::save(
        &evidence.join("scope.json"),
        &json!({"derivedFixture":true,"historical118Resolved":false,"timingUsable":false,
        "workspaceRole":"Linux x64 observed image4f2de/7.0.2, tests Vuebeta10/TS6/vue-tsc3.3.11",
        "historicalRole":"Rc13/provider3.3.13 plus MisskeyVue3.5.28 is separate","phases":7,"compilerOuterCommands":14,"metadataAstNodeHelpers":2,
        "nativeClean":false,"nativeKnownDebt":{"code":5097,"status":1,"file":"src/Parent.vue.ts","line":11,"column":23,"sourceReceipt":"f45/41346587952/artifact13099392298"},
        "debtPolicy":"assert the whole current buffer; removal is a future correctness change, not a permanent public contract or an error filter",
        "loadedNativeFiles":"UNOBSERVED","nativeCompilerSource":"UNJOINED","cacheScope":"same physical context across source mutation/repair/restore"}),
    );
    // Every source, complete stream, native config/body/status and AST observation is durable before semantics.
    assert_eq!(providers["vue"]["version"], "3.6.0-beta.10");
    assert_eq!(providers["reactivity"]["version"], "3.6.0-beta.10");
    assert_eq!(providers["typescript"]["version"], "6.0.3");
    assert_eq!(providers["vueTsc"]["version"], "3.3.11");
    assert_eq!(providers["languageCore"]["version"], "3.3.11");
    assert_eq!(providers["native"]["sha256"], NATIVE_IMAGE);
    assert_eq!(
        providers["native"]["manifest"]["package"]["version"],
        "7.0.2"
    );
    assert_eq!(geometry["typescript"], "6.0.3");
    assert_eq!(geometry["rows"].as_array().unwrap().len(), 7);
    let baseline: Value = serde_json::from_slice(&observations[0].3.stdout).unwrap();
    for (index, (phase, kind, source, cli, js, calls)) in observations.iter().enumerate() {
        let bad = !kind.is_empty();
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
        let (_, expected_js) = oracles::expected_public(kind, source.as_str());
        assert_eq!(js.stdout, expected_js.as_bytes(), "{phase}");
        oracles::assert_public(
            &serde_json::from_slice::<Value>(&cli.stdout).unwrap(),
            calls,
            kind,
            source.as_str(),
            &baseline,
        );
        assert_eq!(geometry["rows"][index]["phase"], *phase);
        oracles::assert_native(calls, &geometry["rows"][index], kind);
        for call in calls.as_array().unwrap() {
            let request = &call["request"];
            assert_eq!(request["executable"], json!(runtime));
            assert_eq!(request["imageBefore"], NATIVE_IMAGE);
            assert_eq!(
                request["node"]["executable"],
                providers["node"]["executable"]
            );
            assert_eq!(request["node"]["version"], providers["node"]["version"]);
            match request["kind"].as_str().unwrap() {
                "main-cli" => {
                    let argv = request["argv"].as_array().unwrap();
                    assert_eq!(argv.len(), 9);
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
                            request["config"].clone()
                        ]
                    );
                }
                "option-probe" => assert_eq!(
                    request["argv"],
                    json!(["--pretty", "false", "--project", request["config"]])
                ),
                "version" => assert_eq!(request["argv"], json!(["--version"])),
                other => panic!("unknown native invocation {other}: {calls}"),
            }
        }
        assert_eq!(capture::digest(&fs::read(&runtime).unwrap()), NATIVE_IMAGE);
    }
    for (left, right) in [(0, 2), (0, 5), (1, 3), (4, 6)] {
        assert_eq!(observations[left].3.stdout, observations[right].3.stdout);
        assert_eq!(observations[left].3.stderr, observations[right].3.stderr);
        assert_eq!(observations[left].4.stdout, observations[right].4.stdout);
        assert_eq!(observations[left].4.stderr, observations[right].4.stderr);
        assert_eq!(
            oracles::main_call(&observations[left].5)["stdout"],
            oracles::main_call(&observations[right].5)["stdout"]
        );
        assert_eq!(
            oracles::main_call(&observations[left].5)["stderr"],
            oracles::main_call(&observations[right].5)["stderr"]
        );
        assert_eq!(
            oracles::main_call(&observations[left].5)["status"]["status"],
            oracles::main_call(&observations[right].5)["status"]["status"]
        );
    }
}
