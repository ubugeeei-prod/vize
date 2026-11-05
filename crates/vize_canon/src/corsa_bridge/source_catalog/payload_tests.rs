use super::*;
use crate::corsa_bridge::CorsaMaterializedMappingKind;
use crate::corsa_bridge::vue_document::CorsaNativeDocumentBatch;

fn source(root: &Path, name: &str, text: &str) -> CorsaMaterializedSource {
    CorsaMaterializedSource {
        materialized_path: root.join("native").join(name),
        source_path: root.join(name),
        source: text.into(),
        code: text.into(),
        mapping: Default::default(),
        import_source_map: Default::default(),
        mapping_kind: CorsaMaterializedMappingKind::AuthoredIdentity,
    }
}

fn capture_members(mut catalog: CorsaSourceCatalog) -> CorsaSourceCatalog {
    let mut sources = recover_lock(&catalog.sources)
        .values()
        .cloned()
        .collect::<Vec<_>>();
    sources.sort_unstable_by(|left, right| left.materialized_path.cmp(&right.materialized_path));
    catalog.native_members = Arc::new(sources);
    catalog
}

fn batch(catalog: &CorsaSourceCatalog, root: &Path) -> CorsaNativeDocumentBatch {
    let mut documents = CorsaNativeDocumentBatch::default();
    catalog.extend_native_project_document_batch(root, &mut documents);
    documents
}

#[test]
fn repeated_native_batches_share_only_unchanged_payloads_and_keep_complete_old_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let a = "// 雪😀\r\nexport const a = 1;\r\n";
    let b = "export const b = 'before';\n";
    let edited_b = "export const b = 'longer 😀 unsaved';\n";
    let catalog = capture_members(
        CorsaSourceCatalog::default()
            .include(vec![source(root, "A.ts", a), source(root, "B.ts", b)]),
    );
    let first = batch(&catalog, root);
    let repeated = batch(&catalog, root);
    let expected = vec![
        (
            crate::file_uri::path_to_file_uri(&root.join("native/A.ts")),
            a.into(),
        ),
        (
            crate::file_uri::path_to_file_uri(&root.join("native/B.ts")),
            b.into(),
        ),
    ];
    assert_eq!(first.to_vec(), expected);
    assert_eq!(repeated.to_vec(), expected);
    for (old, current) in first
        .shared_additions_for_test()
        .iter()
        .zip(repeated.shared_additions_for_test())
    {
        assert!(Arc::ptr_eq(old, current));
    }
    let edited = capture_members(catalog.include(vec![source(root, "B.ts", edited_b)]));
    let changed = batch(&edited, root);
    assert_eq!(
        changed.to_vec(),
        vec![
            (expected[0].0.clone(), a.into()),
            (expected[1].0.clone(), edited_b.into()),
        ]
    );
    assert!(Arc::ptr_eq(
        &first.shared_additions_for_test()[0],
        &changed.shared_additions_for_test()[0]
    ));
    assert!(!Arc::ptr_eq(
        &first.shared_additions_for_test()[1],
        &changed.shared_additions_for_test()[1]
    ));
    assert_eq!(first.to_vec(), expected);
    let live = FxHashSet::from_iter([root.join("native/A.ts")]);
    let retired = capture_members(edited.retaining_live_files(&live, &FxHashSet::default()));
    assert_eq!(batch(&retired, root).to_vec(), vec![expected[0].clone()]);
    assert_eq!(recover_lock(&retired.native_payloads).0.len(), 1);
    // A frozen old query cannot grow the latest catalog's memo again.
    assert_eq!(batch(&catalog, root).to_vec(), expected);
    assert_eq!(recover_lock(&retired.native_payloads).0.len(), 1);
    assert_eq!(first.to_vec(), expected);
}

#[test]
fn an_owned_native_batch_does_not_keep_authored_source_or_mapping_facts_alive() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let text = "export const label = '雪😀';\r\n";
    let catalog =
        capture_members(CorsaSourceCatalog::default().include(vec![source(root, "A.ts", text)]));
    let identity = Arc::downgrade(&catalog.get(&root.join("native/A.ts")).unwrap());
    let documents = batch(&catalog, root);
    drop(catalog);
    assert!(identity.upgrade().is_none());
    assert_eq!(
        documents.to_vec(),
        vec![(
            crate::file_uri::path_to_file_uri(&root.join("native/A.ts")),
            text.into()
        )]
    );
}
