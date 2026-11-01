//! Finite 7.1-only B/C/C/B discriminator; no Maestro pool or production change.
use super::*;
use std::time::Instant;

mod proc;
use proc::{child_pids, native_owners, sample};

struct Owners {
    a: CorsaProjectClient,
    b: Option<CorsaProjectClient>,
    b_uri: String,
}

impl Owners {
    fn new(native: &Path, root: &Path, b_uri: &str, baseline: bool, attachment_name: &str) -> Self {
        assert!(child_pids(std::process::id()).is_empty());
        let mut a = CorsaProjectClient::new_for_editor_workspace(native.to_str(), root).unwrap();
        let a_life = native_owners(root, native, 1).pop().unwrap();
        let mut b = baseline
            .then(|| CorsaProjectClient::new_for_editor_workspace(native.to_str(), root).unwrap());
        for client in std::iter::once(&mut a).chain(b.iter_mut()) {
            assert!(client.project_session_attached_to_editor);
            assert_eq!(
                client.project_session().unwrap().client().dialect(),
                Some(ApiDialect::DerivedSnapshots)
            );
        }
        let result = Self {
            a,
            b,
            b_uri: b_uri.into(),
        };
        let lives = native_owners(root, native, if baseline { 2 } else { 1 });
        assert!(lives.contains(&a_life));
        let b_life = if baseline {
            lives.iter().find(|life| **life != a_life).unwrap()
        } else {
            &a_life
        };
        result.record_attachments(attachment_name, &a_life, b_life);
        result
    }

    fn route(&mut self, uri: &str) -> &mut CorsaProjectClient {
        if uri == self.b_uri.as_str() {
            self.b.as_mut().unwrap_or(&mut self.a)
        } else {
            &mut self.a
        }
    }

    fn publish(&mut self, a: Option<(&str, &str)>, b: Option<(&str, &str)>) {
        if let Some(second) = self.b.as_mut() {
            publish(&mut self.a, a, None);
            publish(second, None, b);
        } else {
            publish(&mut self.a, a, b);
        }
    }

    fn record_attachments(
        &self,
        name: &str,
        a_life: &control::ProcessLife,
        b_life: &control::ProcessLife,
    ) {
        let attachment = |client: &CorsaProjectClient| {
            serde_json::to_value(
                &client
                    .editor_lsp
                    .as_ref()
                    .unwrap()
                    .configured_api
                    .as_ref()
                    .unwrap()
                    .session,
            )
            .unwrap()
        };
        let a = attachment(&self.a);
        let b = self.b.as_ref().map(attachment).unwrap_or_else(|| a.clone());
        if self.b.is_some() {
            assert_ne!(a["pipe"], b["pipe"]);
        } else {
            assert_eq!(a, b);
        }
        // Called once per epoch; preserve the full actual attachment DTOs.
        record(
            name,
            &json!({"a":a,"b":b,
            "aNative":{"pid":a_life.pid,"birth":a_life.birth,"executable":a_life.executable},
            "bNative":{"pid":b_life.pid,"birth":b_life.birth,"executable":b_life.executable},
            "join":"only child after actual A constructor; sole new child after actual B constructor; full identities rechecked"}),
        );
    }

    fn shutdown(&mut self) {
        self.a.shutdown().unwrap();
        if let Some(second) = self.b.as_mut() {
            second.shutdown().unwrap();
        }
    }
}

fn resource_provider() -> std::path::PathBuf {
    use std::io::Read;
    assert!(control::enabled());
    let native = control::runtime();
    let mut file = fs::File::open(&native).unwrap();
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let bytes = file.read(&mut buffer).unwrap();
        if bytes == 0 {
            break;
        }
        digest.update(&buffer[..bytes]);
    }
    let image = digest
        .finalize()
        .iter()
        .map(|byte| cstr!("{byte:02x}"))
        .collect::<String>();
    let version = Command::new(&native).arg("--version").output().unwrap();
    record(
        "provider",
        &json!({"path":native,"sha256":image,"versionCommand":{"argv":["--version"],"status":version.status.code(),"stdout":version.stdout,"stderr":version.stderr}}),
    );
    assert_eq!(
        image,
        "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70"
    );
    assert!(version.status.success());
    assert_eq!(version.stdout, b"Version 7.1.0-beta\n");
    assert!(version.stderr.is_empty());
    native
}

fn timed_packet(owners: &mut Owners, name: &str, uri: &str, config: &Path, column: u32) -> Value {
    let client = owners.route(uri);
    let observed = client.logical_configuration_for_test(uri);
    record(
        cstr!("{name}-editor-project").as_str(),
        &json!({"uri":uri,"expectedConfig":config,"observed":observed}),
    );
    assert_eq!(observed.unwrap(), config);
    assert!(!client.can_use_api_for_uri(uri));
    let documents = client.document_texts.clone();
    let start = Instant::now();
    let diagnostics = client.diagnostics_via_editor_lsp(uri, &documents);
    let diagnostics_ns = start.elapsed().as_nanos();
    record(cstr!("{name}-diagnostics").as_str(), &json!(diagnostics));
    let diagnostics = diagnostics.unwrap();
    let start = Instant::now();
    let hover = client.hover_raw(uri, 1, 2);
    let hover_ns = start.elapsed().as_nanos();
    record(cstr!("{name}-hover").as_str(), &json!(hover));
    let hover = hover.unwrap();
    let start = Instant::now();
    let completion = client.completion_raw(uri, 1, column);
    let completion_ns = start.elapsed().as_nanos();
    record(cstr!("{name}-completion").as_str(), &json!(completion));
    let completion = completion.unwrap();
    let value = json!({"uri":uri,"diagnostics":diagnostics,"hover":hover,"completion":completion});
    record(name, &value);
    record(
        cstr!("resource-timing-{name}").as_str(),
        &json!({"diagnosticsNs":diagnostics_ns,"hoverNs":hover_ns,"completionNs":completion_ns,
        "scope":"synchronous actual individual production calls; no strace, config attestation/serialization outside call timers"}),
    );
    value
}

#[test]
#[ignore = "private four-process native resource discriminator; exact native 7.1 Actions only"]
fn native_two_config_owner_resource_discriminator() {
    let native = resource_provider();
    let mode = std::env::var("VIZE_RESOURCE_MODE").unwrap();
    assert!(mode == "baseline" || mode == "candidate");
    let baseline = mode == "baseline";
    let count = if baseline { 2 } else { 1 };
    let root = std::path::PathBuf::from(std::env::var_os("VIZE_RESOURCE_INPUT_ROOT").unwrap())
        .canonicalize()
        .unwrap();
    let a_source = root.join("a/source.ts");
    let a = root.join("tsconfig.json");
    let b = root.join("b/tsconfig.json");
    assert_eq!(fs::read(&a_source).unwrap(), A_BAD.as_bytes());
    assert_eq!(
        fs::read(root.join("b/source.ts")).unwrap(),
        B_GOOD.as_bytes()
    );
    let a_config_before = fs::read(&a).unwrap();
    let b_config_before = fs::read(&b).unwrap();
    let a_uri = crate::file_uri::path_to_file_uri(&a_source);
    let b_uri = crate::file_uri::path_to_file_uri(&root.join("b/source.ts"));
    assert!(union(Some((&a_uri, A_BAD)), Some((&a_uri, A_GOOD))).is_err());
    sample("root-before", &root, &native, 0);
    let startup = Instant::now();
    let mut clients = Owners::new(&native, &root, &b_uri, baseline, "resource-attachments");
    clients.publish(Some((&a_uri, A_BAD)), Some((&b_uri, B_GOOD)));
    let a_metadata = config_metadata(
        clients.route(&a_uri),
        "production-a-initial-config",
        &a,
        &a_uri,
    );
    let b_metadata = config_metadata(
        clients.route(&b_uri),
        "production-b-initial-config",
        &b,
        &b_uri,
    );
    let a_bad = timed_packet(&mut clients, "production-a-error", &a_uri, &a, 7);
    let b_before = timed_packet(&mut clients, "production-b-before", &b_uri, &b, 6);
    assert_diagnostics(
        &a_bad,
        expected_error(19, "Type 'string' is not assignable to type 'number'."),
    );
    assert_diagnostics(&b_before, clean());
    record(
        "resource-first-usable",
        &json!({"wallNs":startup.elapsed().as_nanos(),"mode":mode,
        "scope":"fresh client construction through both full valid packets and startup config controls; includes raw packet writing, excludes fixture/provider hash and proc samples; OS page cache uncontrolled"}),
    );
    let first_owners = sample("initial", &root, &native, count);
    clients.publish(Some((&a_uri, A_NULL)), Some((&b_uri, B_NULL)));
    let a_null_metadata = config_metadata(
        clients.route(&a_uri),
        "production-a-null-config",
        &a,
        &a_uri,
    );
    let b_null_metadata = config_metadata(
        clients.route(&b_uri),
        "production-b-null-config",
        &b,
        &b_uri,
    );
    assert_diagnostics(
        &timed_packet(&mut clients, "production-a-strict-null", &a_uri, &a, 7),
        expected_error(19, "Type 'null' is not assignable to type 'number'."),
    );
    assert_diagnostics(
        &timed_packet(&mut clients, "production-b-nonstrict-null", &b_uri, &b, 6),
        clean(),
    );
    drop(a_null_metadata);
    drop(b_null_metadata);
    assert_eq!(sample("strict-null", &root, &native, count), first_owners);
    clients.publish(Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD)));
    let original_api = a_metadata.client().clone();
    let original_snapshot = a_metadata.snapshot_lease();
    let original_project = a_metadata.project().id.clone();
    let original_type = held_type(&original_api, &original_snapshot, &original_project, &a_uri);
    record(
        "production-original-readonly-type",
        &json!({"scope":"original readonly lease, not current editor world","type":original_type,"project":original_project,"snapshot":original_snapshot.handle}),
    );
    assert_eq!(original_type, "number");
    assert_diagnostics(
        &timed_packet(&mut clients, "production-a-repaired", &a_uri, &a, 7),
        clean(),
    );
    assert_unchanged_semantic_packet(
        &timed_packet(&mut clients, "production-b-after-a-repair", &b_uri, &b, 6),
        &b_before,
    );
    assert_eq!(sample("repaired", &root, &native, count), first_owners);
    clients.publish(Some((&a_uri, PRODUCTION_NEXT)), Some((&b_uri, B_GOOD)));
    let current = timed_packet(&mut clients, "production-current-boolean", &a_uri, &a, 7);
    assert_diagnostics(&current, clean());
    assert_eq!(
        current["hover"],
        json!({"contents":{"kind":"markdown","value":"```typescript\nconst answer: boolean\n```\n"},"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":6}}})
    );
    let old_type = held_type(&original_api, &original_snapshot, &original_project, &a_uri);
    record(
        "production-distinct-worlds",
        &json!({"oldReadonlyType":old_type,"currentEditorHover":current["hover"],"currentApiAllowed":clients.route(&a_uri).can_use_api_for_uri(&a_uri)}),
    );
    assert_eq!(old_type, "number");
    assert_eq!(
        sample("current-boolean", &root, &native, count),
        first_owners
    );
    drop(a_metadata);
    clients.publish(None, Some((&b_uri, B_GOOD)));
    assert_unchanged_semantic_packet(
        &timed_packet(&mut clients, "production-b-after-a-detach", &b_uri, &b, 6),
        &b_before,
    );
    let detached_type = held_type(&original_api, &original_snapshot, &original_project, &a_uri);
    record(
        "production-held-read-after-detach",
        &json!({"type":detached_type,"scope":"retained readonly lease; entered IPC cancellation not claimed"}),
    );
    assert_eq!(detached_type, "number");
    assert_eq!(sample("a-detached", &root, &native, count), first_owners);
    clients.publish(Some((&a_uri, A_GOOD)), Some((&b_uri, B_BAD)));
    let b_current_metadata = config_metadata(
        clients.route(&b_uri),
        "production-b-edited-config",
        &b,
        &b_uri,
    );
    assert_diagnostics(
        &timed_packet(&mut clients, "production-b-error", &b_uri, &b, 6),
        expected_error(18, "Type 'number' is not assignable to type 'string'."),
    );
    assert_diagnostics(
        &timed_packet(&mut clients, "production-a-reopened", &a_uri, &a, 7),
        clean(),
    );
    assert_eq!(
        sample("a-reopened-b-error", &root, &native, count),
        first_owners
    );
    clients.publish(Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD)));
    let b_clean = timed_packet(&mut clients, "production-b-repaired", &b_uri, &b, 6);
    assert_diagnostics(&b_clean, clean());
    assert_unchanged_semantic_packet(&b_clean, &b_before);
    let issued = completion_item(&b_clean, "toUpperCase");
    let resolved = clients
        .route(&b_uri)
        .completion_resolve_via_editor_lsp(&b_uri, issued.clone());
    record("production-resolve-before-restart", &json!(resolved));
    assert!(resolved.unwrap().is_some());
    drop(original_snapshot);
    drop(original_api);
    drop(b_metadata);
    drop(b_current_metadata);
    assert_eq!(
        sample("repaired-leases-dropped", &root, &native, count),
        first_owners
    );
    clients.shutdown();
    drop(clients);
    for owner in &first_owners {
        control::assert_reaped(owner);
    }
    sample("owners-retired", &root, &native, 0);
    // New epoch uses new record names; no environment mutation in multithreaded Rust.
    let mut replacement = Owners::new(
        &native,
        &root,
        &b_uri,
        baseline,
        "resource-attachments-restarted",
    );
    let next_owners = native_owners(&root, &native, count);
    assert!(next_owners.iter().all(|next| !first_owners.contains(next)));
    replacement.publish(Some((&a_uri, A_GOOD)), Some((&b_uri, B_GOOD)));
    let stale = replacement
        .route(&b_uri)
        .completion_resolve_via_editor_lsp(&b_uri, issued);
    record("production-resolve-stale-after-restart", &json!(stale));
    assert_eq!(stale.unwrap(), None);
    let next_a_metadata = config_metadata(
        replacement.route(&a_uri),
        "production-a-restart-config",
        &a,
        &a_uri,
    );
    let next_b_metadata = config_metadata(
        replacement.route(&b_uri),
        "production-b-restart-config",
        &b,
        &b_uri,
    );
    assert_diagnostics(
        &timed_packet(&mut replacement, "production-a-restart", &a_uri, &a, 7),
        clean(),
    );
    assert_diagnostics(
        &timed_packet(&mut replacement, "production-b-restart", &b_uri, &b, 6),
        clean(),
    );
    assert_eq!(sample("restarted", &root, &native, count), next_owners);
    drop(next_a_metadata);
    drop(next_b_metadata);
    replacement.shutdown();
    drop(replacement);
    for owner in &next_owners {
        control::assert_reaped(owner);
    }
    sample("final", &root, &native, 0);
    assert_eq!(fs::read(&a_source).unwrap(), A_BAD.as_bytes());
    assert_eq!(
        fs::read(root.join("b/source.ts")).unwrap(),
        B_GOOD.as_bytes()
    );
    assert_eq!(fs::read(&a).unwrap(), a_config_before);
    assert_eq!(fs::read(&b).unwrap(), b_config_before);
}
