use super::*;
use vize_l2::resolution::Usage;
use vize_l3::decision::dom::vue::VueReadKind;

#[test]
fn js_ts_const_let_and_var_classify_the_actual_retained_setup_collection_read() {
    let arena = Allocator::default();
    for lang in ["", " lang='ts'"] {
        for (declaration, expected) in [
            ("const", VueReadKind::SetupConst),
            ("let", VueReadKind::SetupLet),
            ("var", VueReadKind::SetupLet),
        ] {
            let source = format!(
                "<script setup{lang}>{declaration} 項目=2;</script><template><div v-for='品 in 項目'/></template>"
            );
            let owner = completed(&arena, &source);
            let original = first(&owner);
            let lower = owner.file().unwrap().for_head_for(original).unwrap();
            let resolution = lower.resolution().unwrap();
            let actual = resolution.collection_occurrence();
            assert_eq!(*actual, resolution.collection());
            let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
            let row = analysis.for_head(original.id().node()).unwrap();
            let read = row.collection_read().unwrap();
            assert!(core::ptr::eq(read.occurrence(), actual));
            assert!(core::ptr::eq(
                read.occurrence(),
                row.resolution().collection_occurrence()
            ));
            assert!(core::ptr::eq(read.binding().file(), analysis.file()));
            assert!(read.binding().same_owner(row.collection()));
            assert_eq!(read.binding().id(), actual.binding);
            assert_eq!(read.occurrence().usage, Usage::Read);
            assert_eq!(read.occurrence().name, "項目");
            assert_eq!(read.kind(), expected);
            assert_eq!(
                row.resolution().collection_authored_span().slice(&source),
                "項目"
            );
            let retained = row
                .resolution()
                .input()
                .operand()
                .syntax()
                .collection()
                .unwrap()
                .unwrap();
            assert_eq!(retained.source().text(), "項目");
            assert!(core::ptr::eq(
                retained.source().authored_root(),
                source.as_str()
            ));
            assert!(core::ptr::eq(
                retained.admitted_expression().unwrap().expression(),
                row.resolution().input().collection()
            ));
            assert_eq!(analysis.tables().nodes.len(), 2);
            assert_eq!(analysis.tables().controls.len(), 1);
            let refusals = analysis.dom().unwrap().unsupported();
            assert_eq!(refusals.len(), 1);
            assert_eq!(refusals[0].reason, DomUnsupported::Operation);
        }
    }
}

#[test]
fn same_name_alias_never_substitutes_for_the_original_enclosing_setup_read() {
    let arena = Allocator::default();
    let owner = completed(
        &arena,
        "<script setup>let item=2;</script><template><div v-for='item in item'/></template>",
    );
    let original = first(&owner);
    let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
    let row = analysis.for_head(original.id().node()).unwrap();
    let read = row.collection_read().unwrap();
    assert_eq!(read.kind(), VueReadKind::SetupLet);
    assert_ne!(read.binding().id(), row.head().value().unwrap().id());
    assert!(read.binding().declaration().is_some());
    assert!(read.binding().template_declaration().is_none());
    assert_eq!(read.occurrence().name, "item");
    assert!(core::ptr::eq(
        read.occurrence(),
        row.resolution().collection_occurrence()
    ));
}

#[test]
fn a_parent_alias_retains_original_custody_and_gets_distinct_access_refusal() {
    let arena = Allocator::default();
    let owner = completed(
        &arena,
        "<script setup>const items=2;</script><template><div v-for='item in items'><span v-for='child in item'/></div></template>",
    );
    let original = first(&owner);
    let [Op::Element(div)] = original.region.ops.as_slice() else {
        panic!("div")
    };
    let [Op::OriginalFor(inner)] = div.children.ops.as_slice() else {
        panic!("inner")
    };
    let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
    let outer_row = analysis.for_head(original.id().node()).unwrap();
    let inner_row = analysis.for_head(inner.id().node()).unwrap();
    assert_eq!(
        outer_row.collection_read().unwrap().kind(),
        VueReadKind::SetupConst
    );
    assert!(inner_row.collection_read().is_none());
    assert_eq!(
        inner_row.collection().id(),
        outer_row.head().value().unwrap().id()
    );
    assert!(inner_row.collection().declaration().is_none());
    let declaration = inner_row.collection().template_declaration().unwrap();
    assert_eq!(declaration.declaration().origin(), original.id());
    assert!(core::ptr::eq(
        inner_row.resolution().input().collection(),
        analysis
            .file()
            .for_head_for(inner)
            .unwrap()
            .resolution()
            .unwrap()
            .input()
            .collection()
    ));
    let refusals = analysis.dom().unwrap().unsupported();
    assert_eq!(refusals.len(), 3);
    assert_eq!(refusals[0].reason, DomUnsupported::Operation);
    assert_eq!(refusals[1].reason, DomUnsupported::ForCollectionAccess);
    assert_eq!(refusals[1].node, inner.id().node());
    assert_eq!(refusals[1].span.slice(analysis.artifact().source()), "item");
    assert_eq!(refusals[2].reason, DomUnsupported::Operation);
    assert_eq!(refusals[2].node, inner.id().node());
}

#[test]
fn foreign_equal_collection_occurrences_and_bindings_never_replace_the_read() {
    let arena = Allocator::default();
    let source =
        "<script setup>const items=2;</script><template><div v-for='item in items'/></template>";
    let owner = completed(&arena, source);
    let foreign = completed(&arena, source);
    let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
    let other = build_native_dom_file_decisions(foreign.view().unwrap()).unwrap();
    let original = first(&owner);
    let row = analysis.for_head(original.id().node()).unwrap();
    let other_row = other.for_head(first(&foreign).id().node()).unwrap();
    let read = row.collection_read().unwrap();
    let foreign_read = other_row.collection_read().unwrap();
    assert_eq!(read.occurrence(), foreign_read.occurrence());
    assert_eq!(read.binding().id(), foreign_read.binding().id());
    assert!(!core::ptr::eq(read.occurrence(), foreign_read.occurrence()));
    assert!(!read.binding().same_owner(foreign_read.binding()));
    assert!(core::ptr::eq(
        read.occurrence(),
        row.resolution().collection_occurrence()
    ));
    assert!(!row.accepts_original(other_row.original()));
}

#[test]
fn movement_preserves_the_normally_owned_occurrence_source_and_read_binding() {
    let arena = Allocator::default();
    let source =
        "<script setup>var items=2;</script><template><div v-for='item in items'/></template>";
    let owner = completed(&arena, source);
    let lower = owner.file().unwrap().for_head_for(first(&owner)).unwrap();
    let pointer = lower.resolution().unwrap().collection_occurrence() as *const _;
    let original_collection = lower.resolution().unwrap().input().collection() as *const _;
    let binding_id = lower.resolution().unwrap().collection().binding;
    let owner = core::hint::black_box(owner);
    let analysis = build_native_dom_file_decisions(owner.view().unwrap()).unwrap();
    let row = analysis.for_head(first(&owner).id().node()).unwrap();
    let read = row.collection_read().unwrap();
    assert!(core::ptr::eq(read.occurrence(), pointer));
    assert!(core::ptr::eq(
        row.resolution().input().collection(),
        original_collection
    ));
    assert_eq!(read.binding().id(), binding_id);
    assert!(core::ptr::eq(read.binding().file(), analysis.file()));
    assert_eq!(read.kind(), VueReadKind::SetupLet);
}

#[test]
fn whole_setup_policy_and_entity_refusals_never_gain_a_collection_read() {
    let arena = Allocator::default();
    for (script, head) in [
        ("const items=[];", "item in items"),
        ("const items={};", "item in items"),
        ("const items=2;", "item in &#105;tems"),
    ] {
        let source =
            format!("<script setup>{script}</script><template><div v-for='{head}'/></template>");
        let mut owner = prepared(&arena, &source);
        {
            let mut walk = owner.begin().unwrap();
            assert!(
                walk.child(walk.selected().children().next().unwrap())
                    .is_err()
            );
        }
        let owner = owner.finish();
        assert!(owner.view().is_err());
        let file = owner.file().unwrap();
        assert_eq!(file.artifact().node_count(), 0);
        assert!(!file.rejected_for_heads().is_empty());
        assert!(matches!(
            build_dom_file_decisions(file),
            Err(DecisionBuildError::IncompleteFile)
        ));
    }
    let mut exported = prepared(
        &arena,
        "<script setup>const items=2;export{items};</script><template><div v-for='item in items'/></template>",
    );
    assert_eq!(
        exported.begin().err().unwrap().kind,
        vize_l2::lang::js::NativeTemplateIssueKind::UnsupportedExport
    );
    let exported = exported.finish();
    assert!(exported.view().is_err());
    let file = exported.file().unwrap();
    assert_eq!(file.artifact().node_count(), 0);
    assert!(
        file.rejected_for_heads().is_empty(),
        "the preflight refusal precedes any head observation"
    );
    assert_eq!(file.template_declarations().count(), 0);
}
