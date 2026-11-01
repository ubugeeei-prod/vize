use super::*;
use crate::{
    batch::{ImportRewriter, generate_vue_document_virtual_ts},
    virtual_ts::{VirtualTsOptions, VizeMapping, VizeSemanticLinkKind},
};
use std::{fs, path::Path};

const INPUT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/private-constructor-return/Props.vue.txt"
));

fn original() -> crate::batch::VueDocumentVirtualTs {
    generate_vue_document_virtual_ts(
        Path::new("Props.vue"),
        INPUT,
        &VirtualTsOptions::default(),
        &ImportRewriter::new(),
        true,
    )
    .unwrap()
}

#[test]
fn actual_pinned_sfc_keeps_whole_output_and_projection_except_owned_return() {
    let original = original();
    let source = original.pre_rewrite_code.as_str();
    let site = super::super::syntax::analyze(source).unwrap();
    let projection = original.mapping.clone();
    let edit = propose(source, &projection, &site, "./private-return.js").unwrap();
    let expected = source.replacen(
        source.get(site.range.clone()).unwrap(),
        "import(\"./private-return.js\").default<Props, {}, $AuthoredProps, $ComponentInstance>",
        1,
    );
    assert_eq!(edit.code, expected);
    assert_eq!(
        original.mapping, projection,
        "original stays available for rollback"
    );
    let mut restored = edit.projection.clone();
    restored.note_generated_replacement(site.range.start, edit.replacement.len(), site.range.len());
    assert_eq!(
        restored, projection,
        "all model fields, row metadata and links survive"
    );
    let rewritten = RewriteResult {
        code: edit.code.clone(),
        source_map: crate::batch::ImportSourceMap::empty(),
    };
    assert!(edit.accepts_rewrite(&rewritten));
}

#[test]
fn authored_or_supplemental_link_overlap_refuses_without_mutation() {
    let original = original();
    let source = original.pre_rewrite_code.as_str();
    let site = super::super::syntax::analyze(source).unwrap();
    let mapped = ProjectionMapping::from_spans(vec![VizeMapping::new(
        site.range.clone(),
        0..site.range.len(),
    )]);
    let before = mapped.clone();
    assert!(propose(source, &mapped, &site, "./private-return.js").is_none());
    assert_eq!(mapped, before);

    let link = VizeSemanticLink {
        source_range: site.range.start..site.range.start,
        target_range: 0..1,
        kind: VizeSemanticLinkKind::VueTemplatePropBinding,
    };
    let mut linked = ProjectionMapping::default();
    linked.set_prop_default_key_links(vec![link]);
    let before = linked.clone();
    assert!(propose(source, &linked, &site, "./private-return.js").is_none());
    assert_eq!(linked, before);
}

#[test]
fn a_real_original_root_declaration_cannot_steal_the_introduced_import() {
    let owner = tempfile::tempdir().unwrap();
    let source_root = owner.path().join("source");
    let virtual_root = owner.path().join("virtual");
    fs::create_dir_all(&source_root).unwrap();
    fs::create_dir_all(&virtual_root).unwrap();
    fs::write(
        source_root.join("private-return.d.ts"),
        "export default interface Authored {}\n",
    )
    .unwrap();
    let original = original();
    let source = original.pre_rewrite_code.as_str();
    let site = super::super::syntax::analyze(source).unwrap();
    let edit = propose(source, &original.mapping, &site, "./private-return.js").unwrap();
    let rewritten = ImportRewriter::new().rewrite_for_virtual_project(
        &edit.code,
        oxc_span::SourceType::ts(),
        (&source_root, &virtual_root),
        Some(&source_root),
    );
    assert!(!edit.accepts_rewrite(&rewritten));
    assert_eq!(original.pre_rewrite_code.as_str(), source);
    let original_again = ImportRewriter::new().rewrite_for_virtual_project(
        source,
        oxc_span::SourceType::ts(),
        (&source_root, &virtual_root),
        Some(&source_root),
    );
    assert_eq!(original_again.code, original.code);
    assert_eq!(original_again.source_map, original.import_source_map);
}
