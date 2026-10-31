//! Same-image old physical-policy control; never an old-source binary claim.

use super::SessionStorage;
use crate::{
    file_uri::{path_to_file_uri, path_to_typescript_name},
    lsp_client::CorsaProjectClient,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, path::PathBuf, process::Command, sync::Arc};

const MAIN: &str = "import { value } from './support';\nimport type { Ref } from 'vue';\nexport const checked: number = value;\nexport declare const kind: Ref<number>;\nimport { value as aliasValue } from '@/support';\nexport const aliasChecked: number = aliasValue;\nexport declare const marker: FixtureMarker;\nimport { value as extensionValue } from './support.ts';\nexport const extensionChecked: number = extensionValue;\n";
const SUPPORT: &str = "export const value = 1;\n";

fn hash(bytes: &[u8]) -> vize_l0::String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect()
}

fn inventory(root: &Path) -> Vec<Value> {
    fn visit(root: &Path, path: &Path, rows: &mut Vec<Value>) {
        let mut children: Vec<_> = fs::read_dir(path).unwrap().map(Result::unwrap).collect();
        children.sort_by_key(|entry| entry.file_name());
        for entry in children {
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if let Ok(target) = fs::read_link(&path) {
                rows.push(json!({"path":relative,"kind":"link","target":target}));
            } else if metadata.is_dir() {
                rows.push(json!({"path":relative,"kind":"directory"}));
                visit(root, &path, rows);
            } else {
                assert!(metadata.is_file());
                rows.push(json!({"path":relative,"kind":"file","bytes":metadata.len(),"sha256":hash(&fs::read(&path).unwrap()),"readonly":metadata.permissions().readonly(),"modified":metadata.modified().unwrap().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos().to_string()}));
            }
        }
    }
    let mut rows = Vec::new();
    visit(root, root, &mut rows);
    rows
}

fn link(source: &Path, target: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target)
        .or_else(|_| junction::create(source, target))
        .unwrap();
}

fn original_storage_policy(root: &Path) -> SessionStorage {
    let mut storage = SessionStorage::new(root).unwrap();
    // Exactly the original ordinary-workspace storage path. This control
    // changes only physical placement, using the same native image and API.
    storage.root = root.join("node_modules/.vize/corsa-overlay");
    fs::create_dir_all(&storage.root).unwrap();
    storage
}

fn retain(capture: &Path, policy: &str, phase: &str, stage: &str, value: &Value) {
    let path = capture.join(vize_l0::cstr!("{policy}-{phase}-{stage}.json").as_str());
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(file, value).unwrap();
}

fn run(root: &Path, native: &Path, old_policy: bool, capture: &Path) -> Value {
    let virtual_uri = path_to_file_uri(&root.join("src/App.vue.ts"));
    let support_uri = path_to_file_uri(&root.join("src/support.ts"));
    let mut client = CorsaProjectClient::new_for_editor_workspace(native.to_str(), root).unwrap();
    if old_policy {
        client.overlay_storage = Arc::new(original_storage_policy(root));
    }
    client
        .did_open_batch_fast(&[(&virtual_uri, MAIN), (&support_uri, SUPPORT)])
        .unwrap();
    let naturally_materialized = client.materialized_project_session;
    if !naturally_materialized {
        client.activate_materialized_project_session().unwrap();
    }
    assert!(client.materialized_project_session);
    assert_eq!(client.cwd, root);
    assert_eq!(client.project_root, root);
    let materialized_root = client.overlay_storage.root().to_path_buf();
    let generated_config = fs::read(materialized_root.join("tsconfig.json")).unwrap();
    let parsed: Value = serde_json::from_slice(&generated_config).unwrap();
    assert_eq!(
        parsed["extends"],
        root.join("tsconfig.json").to_string_lossy().as_ref()
    );
    assert_eq!(
        parsed["compilerOptions"],
        json!({"allowImportingTsExtensions":true,"noEmit":true})
    );
    let policy = if old_policy { "original" } else { "private" };
    let mut rows = Vec::new();
    for (phase, text) in [
        ("valid", SUPPORT),
        ("dependency-error", "export const value = 'bad';\n"),
        ("dependency-repair", "export const value = 2;\n"),
        ("dependency-close", SUPPORT),
    ] {
        if phase == "dependency-close" {
            client.did_close(&support_uri).unwrap();
        } else {
            client.did_change(&support_uri, text).unwrap();
        }
        let project = client
            .session
            .as_ref()
            .map(|session| session.project().clone());
        let mut prefix = json!({"phase":phase,"project":project,"materializedRoot":materialized_root,"generatedConfig":parsed});
        let diagnostics = client
            .request_diagnostics_full(&virtual_uri)
            .map(|fetch| json!({"usedCache":fetch.used_cache,"items":fetch.diagnostics}));
        prefix["diagnosticsResult"] = serde_json::to_value(&diagnostics).unwrap();
        prefix["projectAfterDiagnostics"] = json!(
            client
                .session
                .as_ref()
                .map(|session| session.project().clone())
        );
        retain(capture, policy, phase, "diagnostics", &prefix);
        let diagnostics = diagnostics.unwrap();
        let hover = client.hover_raw(&virtual_uri, 2, 32);
        prefix["hoverResult"] = serde_json::to_value(&hover).unwrap();
        prefix["projectAfterHover"] = json!(
            client
                .session
                .as_ref()
                .map(|session| session.project().clone())
        );
        retain(capture, policy, phase, "hover", &prefix);
        let hover = hover.unwrap();
        let definition = client.definition_raw(&virtual_uri, 2, 32);
        prefix["definitionResult"] = serde_json::to_value(&definition).unwrap();
        prefix["projectAfterDefinition"] = json!(
            client
                .session
                .as_ref()
                .map(|session| session.project().clone())
        );
        retain(capture, policy, phase, "definition", &prefix);
        let definition = definition.unwrap();
        rows.push(json!({"phase":phase,"diagnostics":diagnostics["items"],"hover":hover,"definition":definition,"project":project}));
        assert_eq!(diagnostics["usedCache"], false);
        assert_eq!(
            fs::read(root.join("src/support.ts")).unwrap(),
            SUPPORT.as_bytes()
        );
    }
    let shutdown = client.shutdown();
    retain(
        capture,
        policy,
        "shutdown",
        "result",
        &json!({"shutdown":shutdown,"rows":rows,"materializedRoot":materialized_root,"generatedConfig":parsed}),
    );
    shutdown.unwrap();
    drop(client);
    if !old_policy {
        assert!(!materialized_root.exists());
    }
    json!({"policy":if old_policy {"original-physical-root-control"} else {"private-session-owner"},"naturallyMaterialized":naturally_materialized,"materializedRoot":materialized_root,"generatedConfig":parsed,"rows":rows,"shutdown":"complete"})
}

fn positive_answers(packet: &Value, support_uri: &str) {
    let rows = packet["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    for row in rows {
        let checked_error = row["diagnostics"].as_array().unwrap().iter().any(|diagnostic| {
            diagnostic["code"] == 2322 && diagnostic["range"] == json!({"start":{"line":2,"character":13},"end":{"line":2,"character":20}})
        });
        assert_eq!(
            checked_error,
            row["phase"] == "dependency-error",
            "checking must observe the dirty dependency and its repair/close"
        );
        assert!(!row["hover"].is_null());
        assert!(row["hover"].to_string().contains("value"));
        let targets = row["definition"].as_array().unwrap();
        assert!(!targets.is_empty());
        assert!(
            targets
                .iter()
                .all(|target| target["uri"] == support_uri || target["targetUri"] == support_uri)
        );
    }
}

fn relocated_messages(packet: &Value, destination: &Value) -> (Value, Vec<Value>) {
    let from = path_to_typescript_name(Path::new(packet["materializedRoot"].as_str().unwrap()));
    let to = path_to_typescript_name(Path::new(destination["materializedRoot"].as_str().unwrap()));
    let mut projected = packet["rows"].clone();
    let mut substitutions = Vec::new();
    for (row_index, row) in projected.as_array_mut().unwrap().iter_mut().enumerate() {
        for (index, diagnostic) in row["diagnostics"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
        {
            // Only single-quoted physical paths in the observed diagnostic
            // message field may relocate. Every other field stays byte-exact.
            let before = diagnostic["message"].as_str().unwrap();
            let mut after = vize_l0::String::default();
            for (part_index, part) in before.split('\'').enumerate() {
                if part_index > 0 {
                    after.push('\'');
                }
                let suffix = part.strip_prefix(from.as_str());
                if part_index % 2 == 1
                    && let Some(suffix) =
                        suffix.filter(|rest| rest.is_empty() || rest.starts_with('/'))
                {
                    after.push_str(&to);
                    after.push_str(suffix);
                } else {
                    after.push_str(part);
                }
            }
            if before != after.as_str() {
                substitutions.push(json!({"field":["rows",row_index,"diagnostics",index,"message"],"before":before,"after":after}));
                diagnostic["message"] = json!(after);
            }
        }
    }
    (projected, substitutions)
}

fn relocation_controls() {
    let from = std::env::temp_dir().join("old-owned-root");
    let to = std::env::temp_dir().join("new-owned-root");
    let from_name = path_to_typescript_name(&from);
    let to_name = path_to_typescript_name(&to);
    let before = vize_l0::cstr!(
        "File '{from_name}/src/a.ts'; sibling '{from_name}-sibling/a.ts'; unquoted {from_name}/a.ts"
    );
    let after = vize_l0::cstr!(
        "File '{to_name}/src/a.ts'; sibling '{from_name}-sibling/a.ts'; unquoted {from_name}/a.ts"
    );
    let packet = json!({"materializedRoot":from,"rows":[{"diagnostics":[{"code":6059,"message":before,"unknown":from_name}],"hover":null,"definition":[]}]});
    let destination = json!({"materializedRoot":to});
    let (projected, substitutions) = relocated_messages(&packet, &destination);
    let mut expected = packet["rows"].clone();
    expected[0]["diagnostics"][0]["message"] = json!(after);
    assert_eq!(projected, expected);
    assert_eq!(substitutions.len(), 1);
    expected[0]["diagnostics"]
        .as_array_mut()
        .unwrap()
        .push(json!({"code":6059,"message":"additional rootDir error"}));
    assert_ne!(
        projected, expected,
        "an additional diagnostic cannot relocate away"
    );
    assert_eq!(
        packet["rows"][0]["diagnostics"][0]["message"],
        json!(before)
    );
}

#[test]
#[ignore = "actual locked native provider; same-job Linux and Windows Actions"]
fn actual_native_old_and_private_roots_preserve_complete_configured_overlay_answers() {
    relocation_controls();
    let capture = PathBuf::from(
        std::env::var_os("VIZE_PRIVATE_OVERLAY_CAPTURE").expect("owned capture required"),
    );
    fs::create_dir_all(&capture).unwrap();
    let native = PathBuf::from(std::env::var_os("CORSA_PATH").expect("physical provider required"))
        .canonicalize()
        .unwrap();
    let version = Command::new(&native).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(version.stdout, b"Version 7.0.2\n");
    assert!(version.stderr.is_empty());
    let native_hash = hash(&fs::read(&native).unwrap());
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let vue = [
        repo.join("node_modules/vue"),
        repo.join("npm/builder/vite/node_modules/vue"),
    ]
    .into_iter()
    .find(|path| path.join("package.json").is_file())
    .expect("original installed Vue package required")
    .canonicalize()
    .unwrap();
    let vue_manifest_hash = hash(&fs::read(vue.join("package.json")).unwrap());
    let source = std::env::var("EXPECTED_SOURCE").unwrap();
    assert_eq!(option_env!("EXPECTED_SOURCE"), Some(source.as_str()));
    let actual_source = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(actual_source.status.success());
    assert_eq!(
        std::str::from_utf8(&actual_source.stdout).unwrap().trim(),
        source
    );
    let test_image = std::env::current_exe().unwrap();
    fs::write(capture.join("identity.json"), serde_json::to_vec_pretty(&json!({"schema":1,"scope":"same-image physical storage policy differential, not an old-source binary or published-product proof","source":source,"native":{"path":native,"sha256":native_hash,"versionStdout":version.stdout},"libraryImage":{"path":test_image,"sha256":hash(&fs::read(&test_image).unwrap())},"vuePackage":{"path":vue,"manifestSha256":vue_manifest_hash}})).unwrap()).unwrap();
    let mut comparisons = Vec::new();
    for root_dir in ["src", "."] {
        for composite in [false, true] {
            let work = tempfile::tempdir().unwrap();
            let root = work.path().canonicalize().unwrap();
            fs::create_dir(root.join("src")).unwrap();
            fs::create_dir_all(root.join("types/fixture")).unwrap();
            fs::write(
                root.join("types/fixture/index.d.ts"),
                "interface FixtureMarker { marker: 'authored' }\n",
            )
            .unwrap();
            fs::create_dir(root.join("node_modules")).unwrap();
            link(&vue, &root.join("node_modules/vue"));
            fs::write(root.join("src/support.ts"), SUPPORT).unwrap();
            fs::write(root.join("src/main.ts"), "export const member = true;\n").unwrap();
            fs::write(
                root.join("src/App.vue"),
                "<template>authored host</template>\n",
            )
            .unwrap();
            let config = serde_json::to_vec_pretty(&json!({"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"Bundler","noEmit":true,"rootDir":root_dir,"composite":composite,"baseUrl":".","paths":{"@/*":["src/*"]},"typeRoots":["types"]},"include":["src/**/*"]})).unwrap();
            fs::write(root.join("tsconfig.json"), &config).unwrap();
            let label = vize_l0::cstr!(
                "root-{}-composite-{composite}",
                if root_dir == "." { "project" } else { "src" }
            );
            let phase_capture = capture.join(label.as_str());
            fs::create_dir(&phase_capture).unwrap();
            let old = run(&root, &native, true, &phase_capture);
            fs::remove_dir_all(root.join("node_modules/.vize")).unwrap();
            let before = inventory(&root);
            let private = run(&root, &native, false, &phase_capture);
            let after = inventory(&root);
            let (relocated, substitutions) = relocated_messages(&old, &private);
            let retained = json!({"rootDir":root_dir,"composite":composite,"authoredConfig":serde_json::from_slice::<Value>(&config).unwrap(),"originalPhysicalPolicy":old,"privateOwner":private,"authoredBefore":before,"authoredAfter":after,"comparison":{"scope":"only exact single-quoted physical mirror paths inside diagnostic message fields relocate; all other keys, values, diagnostic codes, counts and order remain exact","originalRowsAtPrivateRoot":relocated,"substitutions":substitutions}});
            fs::write(
                capture.join(vize_l0::cstr!("{label}.json").as_str()),
                serde_json::to_vec_pretty(&retained).unwrap(),
            )
            .unwrap();
            let support_uri = path_to_file_uri(&root.join("src/support.ts"));
            assert_eq!(fs::read(root.join("tsconfig.json")).unwrap(), config);
            comparisons.push((label, old, private, relocated, before, after, support_uri));
        }
    }
    // Capture every rootDir/composite role before any cross-role assertion.
    assert_eq!(comparisons.len(), 4);
    for (label, old, private, relocated, before, after, support_uri) in comparisons {
        assert_eq!(
            before, after,
            "private role changed the complete authored inventory: {label}"
        );
        positive_answers(&old, &support_uri);
        positive_answers(&private, &support_uri);
        assert_eq!(old["rows"].as_array().unwrap().len(), 4);
        assert_eq!(private["rows"].as_array().unwrap().len(), 4);
        for (old, private) in relocated
            .as_array()
            .unwrap()
            .iter()
            .zip(private["rows"].as_array().unwrap())
        {
            assert_eq!(old["phase"], private["phase"]);
            for key in ["diagnostics", "hover", "definition"] {
                assert_eq!(old[key], private[key], "whole {key} mismatch: {label}");
            }
        }
    }
    assert_eq!(hash(&fs::read(&native).unwrap()), native_hash);
    assert_eq!(
        hash(&fs::read(vue.join("package.json")).unwrap()),
        vue_manifest_hash
    );
}
