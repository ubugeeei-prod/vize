//! Same-image old policy and authored logical-disk controls; no old binary credit.

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

mod logical_control;
use logical_control::inventory;

fn hash(bytes: &[u8]) -> vize_l0::String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect()
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
    storage.root = crate::file_uri::file_uri_to_path(&path_to_file_uri(
        &root.join("node_modules/.vize/corsa-overlay"),
    ))
    .unwrap();
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

fn run(root: &Path, native: &Path, policy: logical_control::Policy, capture: &Path) -> Value {
    let old_policy = policy == logical_control::Policy::Original;
    policy.prepare(root, MAIN);
    if policy == logical_control::Policy::Disk {
        return logical_control::run_disk(root, native, capture, MAIN, SUPPORT);
    }
    let virtual_uri = path_to_file_uri(&root.join("src/App.vue.ts"));
    let support_uri = path_to_file_uri(&root.join("src/support.ts"));
    let mut client = CorsaProjectClient::new_for_editor_workspace(native.to_str(), root).unwrap();
    if old_policy {
        client.authored_workspace = false;
        client.overlay_storage = Arc::new(original_storage_policy(root));
    }
    client
        .did_open_batch_fast(&[(&virtual_uri, MAIN), (&support_uri, SUPPORT)])
        .unwrap();
    let naturally_materialized = client.materialized_project_session;
    if old_policy && !naturally_materialized {
        client.activate_materialized_project_session().unwrap();
    }
    assert_eq!(client.materialized_project_session, old_policy);
    assert_eq!(client.cwd, root);
    assert_eq!(client.project_root, root);
    let materialized_root = client.editor_lsp_project_root();
    let private_storage = client.overlay_storage.root().to_path_buf();
    let generated_config = fs::read(materialized_root.join("tsconfig.json")).unwrap();
    let parsed: Value = serde_json::from_slice(&generated_config).unwrap();
    if old_policy {
        assert_eq!(
            parsed["extends"],
            path_to_typescript_name(&root.join("tsconfig.json")).as_str()
        );
        assert_eq!(
            parsed["compilerOptions"],
            json!({"allowImportingTsExtensions":true,"noEmit":true})
        );
    }
    let mut rows = Vec::new();
    for (phase, text) in [
        ("valid", SUPPORT),
        ("dependency-error", "export const value = 'bad';\n"),
        ("dependency-repair", "export const value = 2;\n"),
        ("dependency-close", SUPPORT),
    ] {
        policy.write_dependency(root, text);
        let synchronization = if phase == "dependency-close" {
            client.did_close(&support_uri)
        } else {
            client.did_change(&support_uri, text)
        };
        retain(
            capture,
            policy.label(),
            phase,
            "sync",
            &json!({"phase":phase,"synchronization":synchronization,"materializedRoot":materialized_root,"project":client.session.as_ref().map(|session| session.project())}),
        );
        synchronization.unwrap();
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
        retain(capture, policy.label(), phase, "diagnostics", &prefix);
        let diagnostics = diagnostics.unwrap();
        let hover = client.hover_raw(&virtual_uri, 2, 32);
        prefix["hoverResult"] = serde_json::to_value(&hover).unwrap();
        prefix["projectAfterHover"] = json!(
            client
                .session
                .as_ref()
                .map(|session| session.project().clone())
        );
        retain(capture, policy.label(), phase, "hover", &prefix);
        let hover = hover.unwrap();
        let definition = client.definition_raw(&virtual_uri, 2, 32);
        prefix["definitionResult"] = serde_json::to_value(&definition).unwrap();
        prefix["projectAfterDefinition"] = json!(
            client
                .session
                .as_ref()
                .map(|session| session.project().clone())
        );
        retain(capture, policy.label(), phase, "definition", &prefix);
        let definition = definition.unwrap();
        prefix["diagnosingConfiguration"] =
            serde_json::to_value(client.logical_configuration_for_test(&virtual_uri)).unwrap();
        retain(capture, policy.label(), phase, "configuration", &prefix);
        rows.push(json!({"phase":phase,"diagnostics":diagnostics["items"],"hover":hover,"definition":definition,"project":project,"diagnosingConfiguration":prefix["diagnosingConfiguration"]}));
        assert_eq!(diagnostics["usedCache"], false);
        assert_eq!(
            fs::read(root.join("src/support.ts")).unwrap(),
            if policy == logical_control::Policy::Disk {
                text.as_bytes()
            } else {
                SUPPORT.as_bytes()
            }
        );
    }
    let shutdown = client.shutdown();
    retain(
        capture,
        policy.label(),
        "shutdown",
        "result",
        &json!({"shutdown":shutdown,"rows":rows,"materializedRoot":materialized_root,"generatedConfig":parsed}),
    );
    shutdown.unwrap();
    drop(client);
    if !old_policy {
        assert!(!private_storage.exists());
    }
    policy.finish(root, SUPPORT);
    json!({"policy":policy.label(),"naturallyMaterialized":naturally_materialized,"materializedRoot":materialized_root,"generatedConfig":parsed,"rows":rows,"shutdown":"complete"})
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
        let value_type = match row["phase"].as_str().unwrap() {
            "valid" | "dependency-close" => "1",
            "dependency-error" => "\"bad\"",
            "dependency-repair" => "2",
            phase => panic!("unexpected phase: {phase}"),
        };
        assert_eq!(
            row["hover"],
            json!({"contents":{"kind":"markdown","value":vize_l0::cstr!("```typescript\n(alias) const value: {value_type}\n```\n")},"range":{"start":{"line":2,"character":31},"end":{"line":2,"character":36}}})
        );
        let targets = row["definition"].as_array().unwrap();
        assert_eq!(targets.len(), 1);
        let target_uri = targets.first().unwrap()["uri"].as_str().unwrap();
        // Native Windows URIs lowercase/escape the drive. Compare its complete
        // physical identity while retaining the exact whole wire packet below.
        assert_eq!(
            crate::file_uri::file_uri_to_path(target_uri)
                .unwrap()
                .canonicalize()
                .unwrap(),
            crate::file_uri::file_uri_to_path(support_uri)
                .unwrap()
                .canonicalize()
                .unwrap()
        );
        assert_eq!(
            row["definition"],
            json!([{"uri":target_uri,"range":{"start":{"line":0,"character":13},"end":{"line":0,"character":18}}}])
        );
    }
}

#[cfg(windows)]
fn lower_drive(path: &str) -> vize_l0::String {
    match path.as_bytes() {
        [drive, b':', b'/', ..] if drive.is_ascii_alphabetic() => vize_l0::cstr!(
            "{}{}",
            char::from(*drive).to_ascii_lowercase(),
            path.get(1..).unwrap()
        ),
        _ => path.into(),
    }
}

fn relocated_messages(packet: &Value, destination: &Value) -> (Value, Vec<Value>) {
    let from = path_to_typescript_name(Path::new(packet["materializedRoot"].as_str().unwrap()));
    let to = path_to_typescript_name(Path::new(destination["materializedRoot"].as_str().unwrap()));
    let roots = [
        (from.clone(), to.clone()),
        #[cfg(windows)]
        (lower_drive(&from), lower_drive(&to)),
    ];
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
                let replacement = roots.iter().find_map(|(from, to)| {
                    part.strip_prefix(from.as_str())
                        .filter(|rest| rest.is_empty() || rest.starts_with('/'))
                        .map(|suffix| (to, suffix))
                });
                if part_index % 2 == 1
                    && let Some((to, suffix)) = replacement
                {
                    after.push_str(to);
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
    #[cfg(windows)]
    {
        let from_name = lower_drive(&from_name);
        let to_name = lower_drive(&to_name);
        let before = vize_l0::cstr!(
            "File '{from_name}/src/a.ts'; sibling '{from_name}-sibling/a.ts'; unquoted {from_name}/a.ts"
        );
        let after = vize_l0::cstr!(
            "File '{to_name}/src/a.ts'; sibling '{from_name}-sibling/a.ts'; unquoted {from_name}/a.ts"
        );
        let mut native_spelling = packet.clone();
        native_spelling["rows"][0]["diagnostics"][0]["message"] = json!(before);
        native_spelling["rows"][0]["diagnostics"][0]["unknown"] = json!(from_name);
        let (projected, substitutions) = relocated_messages(&native_spelling, &destination);
        let mut expected = native_spelling["rows"].clone();
        expected[0]["diagnostics"][0]["message"] = json!(after);
        assert_eq!(projected, expected);
        assert_eq!(substitutions.len(), 1);
    }
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
    fs::write(capture.join("identity.json"), serde_json::to_vec_pretty(&json!({"schema":1,"scope":"same-provider original physical policy plus independent authored-disk vs logical-overlay packets; not old-source binary or published-product proof","source":source,"native":{"path":native,"sha256":native_hash,"versionStdout":version.stdout},"libraryImage":{"path":test_image,"sha256":hash(&fs::read(&test_image).unwrap())},"vuePackage":{"path":vue,"manifestSha256":vue_manifest_hash}})).unwrap()).unwrap();
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
            let old = run(
                &root,
                &native,
                logical_control::Policy::Original,
                &phase_capture,
            );
            fs::remove_dir_all(root.join("node_modules/.vize")).unwrap();
            let disk = run(
                &root,
                &native,
                logical_control::Policy::Disk,
                &phase_capture,
            );
            let before = inventory(&root);
            let private = run(
                &root,
                &native,
                logical_control::Policy::Logical,
                &phase_capture,
            );
            let after = inventory(&root);
            let retained = json!({"rootDir":root_dir,"composite":composite,"authoredConfig":serde_json::from_slice::<Value>(&config).unwrap(),"originalPhysicalPolicy":old,"logicalDiskControl":disk,"logicalOverlay":private,"authoredBefore":before,"authoredAfter":after,"comparison":{"scope":"complete native logical-disk vs unsaved-logical packets, unchanged authored config; original physical-policy packets retained without filtering or acceptance transfer"}});
            fs::write(
                capture.join(vize_l0::cstr!("{label}.json").as_str()),
                serde_json::to_vec_pretty(&retained).unwrap(),
            )
            .unwrap();
            let support_uri = path_to_file_uri(&root.join("src/support.ts"));
            assert_eq!(fs::read(root.join("tsconfig.json")).unwrap(), config);
            comparisons.push((work, label, old, private, disk, before, after, support_uri));
        }
    }
    // Capture every rootDir/composite role before any cross-role assertion.
    crate::corsa_bridge::overlay_namespace_tests::run();
    assert_eq!(comparisons.len(), 4);
    for (_work, label, old, private, disk, before, after, support_uri) in comparisons {
        assert_eq!(
            before, after,
            "private role changed the complete authored inventory: {label}"
        );
        logical_control::assert_configuration(&disk);
        logical_control::assert_configuration(&private);
        positive_answers(&old, &support_uri);
        positive_answers(&disk, &support_uri);
        positive_answers(&private, &support_uri);
        assert_eq!(old["rows"].as_array().unwrap().len(), 4);
        assert_eq!(private["rows"].as_array().unwrap().len(), 4);
        for (old, private) in disk["rows"]
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
