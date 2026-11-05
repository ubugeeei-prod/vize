use super::*;
use std::time::Duration;

const ORIGINAL: &str = "// 雪😀\r\nexport const value = 'before';\r\n";
const CHANGED: &str = "// 雪😀\r\nexport const value = 'after!';\r\n";

fn fixture() -> (tempfile::TempDir, PathBuf, String, CorsaProjectClient) {
    let root = tempfile::tempdir().unwrap();
    let canonical = root.path().canonicalize().unwrap();
    let path = canonical.join("A.vue.ts");
    std::fs::write(root.path().join("__vize_vue_modules.d.ts"), "export {};\n").unwrap();
    std::fs::write(&path, ORIGINAL).unwrap();
    let uri = crate::file_uri::path_to_file_uri(&path);
    let client = CorsaProjectClient::empty_for_test(canonical);
    (root, path, uri, client)
}

fn input(uri: &str, text: &str) -> CorsaNativeDocumentBatch {
    vec![(uri.into(), text.into())].into()
}

// Unit observations use an explicitly owned, complete physical fixture.
// Public memo admission still requires a successful original Canon/API sync;
// the NoAPI rejection is checked independently below and native Actions cover
// the real admitted transport.
fn certificate(client: &mut CorsaProjectClient, batch: &CorsaNativeDocumentBatch) -> CanonBatch {
    let documents = batch
        .iter()
        .map(|(uri, text)| (uri.as_str(), text.as_str()))
        .collect::<Vec<_>>();
    client.did_open_batch_fast(&documents).unwrap();
    CanonBatch {
        identity: PayloadIdentity::capture(batch),
        texts: client.document_texts.checkpoint(),
        rows: documents
            .iter()
            .map(|(uri, text)| CanonRow {
                path: crate::file_uri::file_uri_to_path(uri).unwrap(),
                desired: client.document_texts.desired_digest(uri, text),
            })
            .collect(),
    }
}

fn replay(
    client: &mut CorsaProjectClient,
    batch: &CorsaNativeDocumentBatch,
    entry: &CanonBatch,
) -> Result<bool, String> {
    let documents = batch
        .iter()
        .map(|(uri, text)| (uri.as_str(), text.as_str()))
        .collect::<Vec<_>>();
    client.reuse_canonical_documents(&documents, entry)
}

#[cfg(unix)]
#[test]
fn settled_unchanged_batches_skip_text_reads_but_repair_same_mtime_same_size_tampering() {
    let (_root, path, uri, mut client) = fixture();
    let batch = input(&uri, ORIGINAL);
    let entry = certificate(&mut client, &batch);
    std::thread::sleep(Duration::from_millis(1100));
    assert!(replay(&mut client, &batch, &entry).unwrap());
    let reads = client.canon_file_cache.exact_reads_for_test();
    assert_eq!(reads, 1);
    for _ in 0..16 {
        assert!(replay(&mut client, &batch, &entry).unwrap());
    }
    assert_eq!(client.canon_file_cache.exact_reads_for_test(), reads);
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    assert_eq!(ORIGINAL.len(), CHANGED.len());
    std::fs::write(&path, CHANGED).unwrap();
    std::fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .unwrap();
    assert!(replay(&mut client, &batch, &entry).unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL.as_bytes());
    assert_eq!(client.canon_file_cache.exact_reads_for_test(), reads + 1);
    assert_eq!(client.document_texts.get(uri.as_str()).unwrap(), ORIGINAL);
}

#[test]
fn a_different_alias_observation_cannot_certify_this_unchanged_batchs_desired_bytes() {
    let (_root, path, uri, mut client) = fixture();
    let alias = uri.replace("A.vue.ts", "%41.vue.ts");
    client
        .document_texts
        .insert(alias.clone().into(), CHANGED.into());
    let batch = input(&uri, ORIGINAL);
    let entry = certificate(&mut client, &batch);
    std::thread::sleep(Duration::from_millis(1100));
    assert!(replay(&mut client, &batch, &entry).unwrap());
    let checkpoint = client.document_texts.checkpoint();
    let digest = client.document_texts.desired_digest(&alias, CHANGED);
    assert!(
        client
            .canon_file_cache
            .synchronize_with_digest(&path, CHANGED, || digest)
            .unwrap()
    );
    std::thread::sleep(Duration::from_millis(1100));
    assert!(
        !client
            .canon_file_cache
            .synchronize_with_digest(&path, CHANGED, || digest)
            .unwrap()
    );
    assert_eq!(client.document_texts.checkpoint(), checkpoint);
    assert!(replay(&mut client, &batch, &entry).unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), ORIGINAL.as_bytes());
    assert_eq!(client.document_texts.get(alias.as_str()).unwrap(), CHANGED);
}

#[test]
fn retained_mutation_close_reopen_and_a_new_payload_retire_the_previous_batch_certificate() {
    let (_root, path, uri, mut client) = fixture();
    let batch = input(&uri, ORIGINAL);
    client.canon_batches.observed_canonical_sync();
    client.did_open_immutable_batch(&batch).unwrap();
    assert!(
        client.canon_batches.entries.is_empty(),
        "retain-only mode never receives Canon write authority"
    );
    assert!(
        !client.canon_batches.synchronized,
        "a previous successful batch is not admission for this retain-only batch"
    );
    let first = client.document_texts.checkpoint();
    client.document_texts.insert(uri.clone(), CHANGED.into());
    assert_ne!(client.document_texts.checkpoint(), first);
    client.did_open_immutable_batch(&batch).unwrap();
    assert_eq!(client.document_texts.get(uri.as_str()).unwrap(), ORIGINAL);
    client.document_texts.remove(&uri);
    client.did_open_immutable_batch(&batch).unwrap();
    assert_eq!(client.document_texts.get(uri.as_str()).unwrap(), ORIGINAL);
    std::fs::write(&path, CHANGED).unwrap();
    let replacement = input(&uri, CHANGED);
    client.did_open_immutable_batch(&replacement).unwrap();
    assert_eq!(client.document_texts.get(uri.as_str()).unwrap(), CHANGED);
    assert_eq!(std::fs::read(&path).unwrap(), CHANGED.as_bytes());
    client.shutdown().unwrap();
    assert!(client.canon_batches.entries.is_empty());
}

#[test]
fn payload_certificates_are_bounded_and_do_not_keep_native_bytes_alive() {
    let (_root, _path, uri, mut client) = fixture();
    let inputs = (0..12).map(|_| input(&uri, ORIGINAL)).collect::<Vec<_>>();
    for batch in &inputs {
        let entry = certificate(&mut client, batch);
        let documents = batch
            .iter()
            .map(|(uri, text)| (uri.as_str(), text.as_str()))
            .collect::<Vec<_>>();
        client.remember_canon_batch(
            batch,
            &documents,
            entry.rows.into_iter().map(|row| row.path).collect(),
        );
        assert_eq!(Arc::strong_count(&batch.shared), 1);
    }
    assert_eq!(client.canon_batches.entries.len(), BATCH_CAPACITY);
    let weak = inputs.last().unwrap().payload_owner_for_test();
    drop(inputs);
    assert!(weak.upgrade().is_none());
    let batch = input(&uri, ORIGINAL);
    let entry = certificate(&mut client, &batch);
    let documents = batch
        .iter()
        .map(|(uri, text)| (uri.as_str(), text.as_str()))
        .collect::<Vec<_>>();
    client.remember_canon_batch(
        &batch,
        &documents,
        entry.rows.into_iter().map(|row| row.path).collect(),
    );
    assert_eq!(client.canon_batches.entries.len(), 1);
}

#[cfg(unix)]
#[test]
fn a_retargeted_directory_link_is_never_authority_for_a_reused_batch_write() {
    let (_root, _path, _uri, mut client) = fixture();
    let nested = client.project_root.join("nested");
    std::fs::create_dir(&nested).unwrap();
    let path = nested.join("A.vue.ts");
    std::fs::write(&path, ORIGINAL).unwrap();
    let uri = crate::file_uri::path_to_file_uri(&path);
    let batch = input(&uri, ORIGINAL);
    let entry = certificate(&mut client, &batch);
    client.did_open_immutable_batch(&batch).unwrap();
    let external = tempfile::tempdir().unwrap();
    let outside = external.path().join("A.vue.ts");
    std::fs::write(&outside, CHANGED).unwrap();
    std::fs::rename(&nested, client.project_root.join("previous")).unwrap();
    std::os::unix::fs::symlink(external.path(), &nested).unwrap();
    assert_eq!(
        replay(&mut client, &batch, &entry),
        Err("Canon document left its private physical namespace".into())
    );
    client.did_open_immutable_batch(&batch).unwrap();
    assert!(client.canon_batches.entries.is_empty());
    for _ in 0..2 {
        assert_eq!(
            client.synchronize_canon_documents_for_test(&[(uri.as_str(), ORIGINAL)]),
            Err("Canon document left its private physical namespace".into())
        );
        assert_eq!(std::fs::read(&outside).unwrap(), CHANGED.as_bytes());
    }
    // Genuine linked bytes remain a read-only input to the original writer.
    assert_eq!(
        client.synchronize_canon_documents_for_test(&[(uri.as_str(), CHANGED)]),
        Ok(true)
    );
    assert_eq!(std::fs::read(&outside).unwrap(), CHANGED.as_bytes());
}
