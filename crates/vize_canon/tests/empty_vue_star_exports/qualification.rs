//! Exact declared provider roles, selected native roots, and inherited options.

use super::support::{self, Runtime};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub(super) fn native(executable: &Path) -> Value {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let id = format!(
        "runtime-native-{}",
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );
    let mut receipt = json!({"phase":"PENDING_OFFICIAL_NATIVE_QUALIFICATION","executable":executable,
        "wholeExecutableSHA256":support::hash(&std::fs::read(executable).unwrap()),
        "nativeAcceptance":"pending; no native acceptance credit"});
    support::capture(&id, &receipt);
    let platform = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "win32",
        other => panic!("unqualified native platform: {other}"),
    };
    let architecture = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        other => panic!("unqualified native architecture: {other}"),
    };
    let package = executable.parent().unwrap().parent().unwrap();
    let manifest_record = support::observed_custody(&package.join("package.json"));
    receipt["officialPackageManifest"] = manifest_record.clone();
    support::capture(&id, &receipt);
    let manifest: Value =
        serde_json::from_str(manifest_record["wholeUtf8"].as_str().unwrap()).unwrap();
    let version = std::process::Command::new(executable)
        .arg("--version")
        .current_dir(package)
        .output()
        .unwrap();
    receipt["versionProcess"] = json!({"argv":[executable,"--version"],"cwd":package,
        "exitCode":version.status.code(),"wholeStdoutBytes":version.stdout,"wholeStderrBytes":version.stderr});
    receipt["phase"] = json!("OFFICIAL_NATIVE_PROCESS_RETURNED_PENDING_VALIDATION");
    support::capture(&id, &receipt);
    assert_eq!(
        manifest["name"],
        format!("@typescript/typescript-{platform}-{architecture}")
    );
    assert_eq!(manifest["version"], "7.0.2");
    assert_eq!(manifest["author"], "Microsoft Corp.");
    assert_eq!(
        manifest["repository"]["url"],
        "https://github.com/microsoft/TypeScript.git"
    );
    assert_eq!(manifest["os"], json!([platform]));
    assert_eq!(manifest["cpu"], json!([architecture]));
    let binary = if cfg!(windows) { "tsc.exe" } else { "tsc" };
    assert_eq!(
        executable.canonicalize().unwrap(),
        package.join("lib").join(binary).canonicalize().unwrap()
    );
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(version.stderr, Vec::<u8>::new());
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    receipt["nativeAcceptance"] = json!(
        "official installed native7.0.2 package and actual version0/stderr0 accepted; no LSP/API/PID authority credit"
    );
    receipt["phase"] = json!("OFFICIAL_NATIVE_RUNTIME_VALIDATED");
    support::capture(&id, &receipt);
    receipt
}

pub(super) fn provider_reads(runtime: &Runtime) -> Value {
    let roles = support::value("provider-roles.json");
    let mut actual = Vec::new();
    for role in roles["roles"].as_array().unwrap() {
        let package = match role["packageAlias"].as_str().unwrap() {
            "vue" => &runtime.vue,
            "vite" => &runtime.vite,
            unknown => panic!("unexpected qualified package role: {unknown}"),
        };
        let path = package.join(support::relative(role["relativePath"].as_str().unwrap()));
        let record = support::observed_custody(&path);
        let manifest = support::observed_custody(&package.join("package.json"));
        actual.push(json!({"role":role,"actual":record,"wholeManifestRead":manifest}));
    }
    json!(actual)
}

pub(super) fn providers(providers: &Value) {
    let expected_ids = [
        "vite-client-types",
        "vite-manifest",
        "vue-import-types",
        "vue-manifest",
        "vue-require-types",
    ];
    assert_eq!(
        providers
            .as_array()
            .unwrap()
            .iter()
            .map(|record| record["role"]["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        expected_ids
    );
    for provider in providers.as_array().unwrap() {
        let role = &provider["role"];
        let record = &provider["actual"];
        let expected = support::blob(&role["body"]);
        for field in ["bytes", "sha256", "wholeUtf8"] {
            assert_eq!(
                record[field], expected[field],
                "exact {} provider {field}",
                role["id"]
            );
        }
        let manifest = &provider["wholeManifestRead"];
        let manifest: Value =
            serde_json::from_str(manifest["wholeUtf8"].as_str().unwrap()).unwrap();
        if let Some(fields) = role["declaredBy"].as_array() {
            let mut declared = &manifest;
            for field in fields {
                declared = &declared[field.as_str().unwrap()];
            }
            assert_eq!(
                declared,
                &json!(format!("./{}", role["relativePath"].as_str().unwrap())),
                "the exact authenticated manifest must declare this provider role"
            );
        }
    }
}

pub(super) fn configuration(case: &Value, packet: &Value, mirror: &Path) {
    let input = |name| {
        case["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|input| input["file"] == name)
            .unwrap()
    };
    let authored: Value = serde_json::from_str(&support::body(input("tsconfig.json"))).unwrap();
    let strict = match case["id"].as_str().unwrap() {
        "config-strict" | "config-permissive" => {
            assert_eq!(authored["extends"], "./base.json");
            assert!(authored["compilerOptions"].get("strict").is_none());
            let base: Value = serde_json::from_str(&support::body(input("base.json"))).unwrap();
            let strict = case["id"] == "config-strict";
            assert_eq!(base, json!({"compilerOptions":{"strict":strict}}));
            let decoy: Value =
                serde_json::from_str(&support::body(input("tsconfig.decoy.json"))).unwrap();
            assert_eq!(
                decoy,
                json!({"compilerOptions":{"strict":false},"files":[]})
            );
            strict
        }
        _ => {
            assert_eq!(authored["compilerOptions"]["strict"], true);
            true
        }
    };
    let generated: Value =
        serde_json::from_str(packet["generatedConfig"]["wholeUtf8"].as_str().unwrap()).unwrap();
    assert_eq!(generated["compilerOptions"]["strict"], strict);
    assert_eq!(
        packet["nativeEffectiveConfig"]["compilerOptions"]["strict"],
        strict
    );
    assert_eq!(
        Path::new(packet["authoredConfigPath"].as_str().unwrap())
            .file_name()
            .unwrap(),
        "tsconfig.json"
    );
    let actual = packet["nativeEffectiveConfig"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| {
            let path = PathBuf::from(path.as_str().unwrap());
            (if path.is_absolute() {
                path
            } else {
                mirror.join(path)
            })
            .canonicalize()
            .unwrap()
        })
        .collect::<Vec<_>>();
    let mut expected = packet["wholeGeneratedProgram"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            PathBuf::from(file["virtualPath"].as_str().unwrap())
                .canonicalize()
                .unwrap()
        })
        .collect::<Vec<_>>();
    for name in ["__vize_helpers.d.ts", "__vize_vue_modules.d.ts"] {
        expected.push(mirror.join(name).canonicalize().unwrap());
    }
    let mut actual = actual;
    actual.sort();
    expected.sort();
    assert_eq!(
        actual, expected,
        "complete native selected root identities, including duplicates"
    );
}

pub(super) fn loaded_roles(providers: &Value, files: &Value) -> Value {
    let loaded = files
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            PathBuf::from(file["path"].as_str().unwrap())
                .canonicalize()
                .unwrap()
        })
        .collect::<BTreeSet<_>>();
    let mut observed = Vec::new();
    let mut vue_types = Vec::new();
    let mut vite_types = Vec::new();
    for provider in providers.as_array().unwrap() {
        let role = provider["role"]["id"].as_str().unwrap();
        if role == "vue-import-types" || role == "vue-require-types" || role == "vite-client-types"
        {
            let path = PathBuf::from(provider["actual"]["path"].as_str().unwrap())
                .canonicalize()
                .unwrap();
            let present = loaded.get(&path).is_some();
            if present {
                let file = files
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|file| {
                        PathBuf::from(file["path"].as_str().unwrap())
                            .canonicalize()
                            .unwrap()
                            == path
                    })
                    .unwrap();
                for field in ["bytes", "sha256", "wholeUtf8"] {
                    assert_eq!(
                        file[field], provider["actual"][field],
                        "actual loaded role body: {role}"
                    );
                }
                if role == "vite-client-types" {
                    vite_types.push(role);
                } else {
                    vue_types.push(role);
                }
            }
            observed.push(json!({"role":role,"canonicalPath":path,"actuallyLoaded":present}));
        }
    }
    assert_eq!(vite_types, ["vite-client-types"]);
    assert!(
        !vue_types.is_empty(),
        "the actual declared Vue import or require types must be loaded"
    );
    json!(observed)
}
