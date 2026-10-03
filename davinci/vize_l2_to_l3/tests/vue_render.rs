//! Genuine descriptor/Program membership; diagnostic template custody stays distinct.
#[path = "vue_render_support/constants.rs"]
mod vue_render_constants;
#[path = "vue_render_support/properties.rs"]
mod vue_render_properties;
mod vue_render_support;

use vize_l0::{Allocator, id::NodeId};
use vize_l1::embed::Lang;
use vize_l2::resolution::Usage;
use vize_l3::decision::dom::{
    DomUnsupported, ValueKind,
    vue::{VueReadKind, build_vue_render_decisions},
};
use vue_render_support::{Observed, bridge, expression};

#[test]
fn ordered_duplicate_reads_keep_exact_occurrences_bindings_and_owner() {
    let arena = Allocator::default();
    let source = "<script setup>let msg = 'one'; var count = 1;</script><template>{{msg + count + msg}}</template>";
    let observed = Observed::new(&arena, source).unwrap();
    let owner = expression(&arena, source, "msg + count + msg", Lang::Js).unwrap();
    let js = bridge(&arena, source, &owner).unwrap();
    let file = observed.file(&arena, &[js]).unwrap();
    let exposure = observed.exposure(&file).unwrap();
    let analysis = build_vue_render_decisions(&exposure).unwrap();
    assert!(core::ptr::eq(analysis.file(), &file));
    assert!(core::ptr::eq(analysis.artifact(), file.artifact()));
    assert_eq!(analysis.tables().nodes.len(), 1);
    assert!(analysis.dom().unwrap().unsupported().is_empty());
    let row = analysis.expression(NodeId::FIRST).unwrap();
    let resolution = row.resolution();
    let table = resolution.table().unwrap();
    assert_eq!(row.reads().len(), table.occurrences().len());
    assert_eq!(row.value(), ValueKind::FileDependent);
    assert_eq!(
        row.reads()
            .iter()
            .map(|read| read.occurrence().name)
            .collect::<Vec<_>>(),
        ["msg", "count", "msg"]
    );
    assert_eq!(resolution.scope(), Some(exposure.scope()));
    for (read, occurrence) in row.reads().iter().zip(table.occurrences()) {
        assert!(core::ptr::eq(read.occurrence(), occurrence));
        assert!(core::ptr::eq(read.binding().file(), &file));
        assert_eq!(read.binding().id(), occurrence.binding);
        assert_eq!(read.kind(), VueReadKind::SetupLet);
        assert_eq!(occurrence.usage, Usage::Read);
    }
    assert!(core::ptr::eq(
        table.expression().ast,
        owner.expression().unwrap()
    ));
    assert_eq!(owner.diagnostics().count(), 0);
}

#[test]
fn ts_unicode_comments_and_escaped_reference_names_retain_authored_identity() {
    let arena = Allocator::default();
    let source = r"<script setup lang=ts>let \u006dsg = 1, 日本語 = 2;</script><template>{{\u006dsg /*keep*/ + 日本語}}</template>";
    let observed = Observed::new(&arena, source).unwrap();
    let owner = expression(&arena, source, r"\u006dsg /*keep*/ + 日本語", Lang::Ts).unwrap();
    let js = bridge(&arena, source, &owner).unwrap();
    let file = observed.file(&arena, &[js]).unwrap();
    let exposure = observed.exposure(&file).unwrap();
    let analysis = build_vue_render_decisions(&exposure).unwrap();
    let row = analysis.expression(NodeId::FIRST).unwrap();
    assert_eq!(
        row.reads()
            .iter()
            .map(|read| read.occurrence().name)
            .collect::<Vec<_>>(),
        ["msg", "日本語"]
    );
    let first = row.reads().first().unwrap().occurrence();
    let authored = js.authored_span(first.span).unwrap();
    assert_eq!(
        source.get(authored.start as usize..authored.end as usize),
        Some(r"\u006dsg")
    );
    assert_eq!(owner.comments().count(), 1);
    assert_eq!(owner.diagnostics().count(), 0);
}

#[test]
fn literal_rows_have_complete_empty_reads_and_nonnumeric_owner_identity() {
    let arena = Allocator::default();
    let source = "<script setup>let unused;</script><template>{{'x'}}</template>";
    let observed = Observed::new(&arena, source).unwrap();
    let owner = expression(&arena, source, "'x'", Lang::Js).unwrap();
    let js = bridge(&arena, source, &owner).unwrap();
    let first = observed.file(&arena, &[js]).unwrap();
    let second = observed.file(&arena, &[js]).unwrap();
    let exposure = observed.exposure(&first).unwrap();
    let analysis = build_vue_render_decisions(&exposure).unwrap();
    let row = analysis.expression(NodeId::FIRST).unwrap();
    assert!(row.reads().is_empty());
    assert_eq!(row.value(), ValueKind::LiteralConstant);
    assert!(row.resolution().table().unwrap().occurrences().is_empty());
    assert!(core::ptr::eq(row.resolution().file(), &first));
    assert!(!core::ptr::eq(row.resolution().file(), &second));
    assert_eq!(
        first.artifact().node_count(),
        second.artifact().node_count()
    );
}

#[test]
fn writes_and_mixed_membership_never_publish_partial_read_lists() {
    let arena = Allocator::default();
    for (program, text) in [
        ("let msg = 1;", "msg = 2"),
        ("let msg = 1;", "msg++"),
        (
            "let msg = 1; const original = 2; const fixed = original;",
            "msg + fixed",
        ),
        ("const msg = 1;", "msg = 2"),
        ("const msg = 1;", "msg++"),
        ("let msg = 1; function fixed() { return 1; }", "msg + fixed"),
        (
            "let msg = 1; import { fixed } from 'module';",
            "msg + fixed",
        ),
    ] {
        let source = arena.alloc_str(&vize_l0::cstr!(
            "<script setup>{program}</script><template>{{{{{text}}}}}</template>"
        ));
        let observed = Observed::new(&arena, source).unwrap();
        let owner = expression(&arena, source, text, Lang::Js).unwrap();
        let js = bridge(&arena, source, &owner).unwrap();
        let file = observed.file(&arena, &[js]);
        assert!(file.is_some(), "real file case: {program} / {text}");
        let file = file.unwrap();
        let exposure = observed.exposure(&file).unwrap();
        let analysis = build_vue_render_decisions(&exposure).unwrap();
        assert!(analysis.expression(NodeId::FIRST).is_none());
        assert_eq!(
            analysis
                .dom()
                .unwrap()
                .unsupported()
                .iter()
                .map(|row| row.reason)
                .collect::<Vec<_>>(),
            [DomUnsupported::VueReadAccess]
        );
        assert_eq!(analysis.tables().nodes.len(), 1);
    }
}

#[test]
fn zero_reference_nonliteral_does_not_gain_constant_or_read_admission() {
    let arena = Allocator::default();
    let source = "<script setup>let unused;</script><template>{{1 + 2}}</template>";
    let observed = Observed::new(&arena, source).unwrap();
    let owner = expression(&arena, source, "1 + 2", Lang::Js).unwrap();
    let js = bridge(&arena, source, &owner).unwrap();
    let file = observed.file(&arena, &[js]).unwrap();
    let exposure = observed.exposure(&file).unwrap();
    let analysis = build_vue_render_decisions(&exposure).unwrap();
    assert!(analysis.expression(NodeId::FIRST).is_none());
    assert_eq!(
        analysis
            .dom()
            .unwrap()
            .unsupported()
            .iter()
            .map(|row| row.reason)
            .collect::<Vec<_>>(),
        [DomUnsupported::Expression]
    );
}

#[test]
fn ordinary_ref_calls_and_semantic_marker_collisions_never_supply_the_view() {
    let arena = Allocator::default();
    for source in [
        "<script>let msg = 1;</script><template>{{msg}}</template>",
        "<script setup>import { ref } from 'vue'; let msg = ref(0);</script><template>{{msg}}</template>",
        "<script setup>let __proto__ = 1;</script><template>{{__proto__}}</template>",
        r"<script setup>let \u005f_proto__ = 1;</script><template>{{\u005f_proto__}}</template>",
    ] {
        let observed = Observed::new(&arena, source).unwrap();
        let file = observed.file(&arena, &[]).unwrap();
        assert!(file.is_complete());
        assert!(observed.exposure(&file).is_none());
    }
}

#[test]
fn equal_numeric_binding_ids_in_two_real_files_preserve_distinct_read_owners() {
    let arena = Allocator::default();
    let source = "<script setup>let msg = 1;</script><template>{{msg}}</template>";
    let observed = Observed::new(&arena, source).unwrap();
    let owner = expression(&arena, source, "msg", Lang::Js).unwrap();
    let js = bridge(&arena, source, &owner).unwrap();
    let first = observed.file(&arena, &[js]).unwrap();
    let second = observed.file(&arena, &[js]).unwrap();
    let first_view = observed.exposure(&first).unwrap();
    let second_view = observed.exposure(&second).unwrap();
    let first_analysis = build_vue_render_decisions(&first_view).unwrap();
    let second_analysis = build_vue_render_decisions(&second_view).unwrap();
    let first_read = first_analysis
        .expression(NodeId::FIRST)
        .unwrap()
        .reads()
        .first()
        .unwrap();
    let second_read = second_analysis
        .expression(NodeId::FIRST)
        .unwrap()
        .reads()
        .first()
        .unwrap();
    assert_eq!(first_read.binding().id(), second_read.binding().id());
    assert!(!first_read.binding().same_owner(second_read.binding()));
    assert!(!core::ptr::eq(
        first_read.occurrence(),
        second_read.occurrence()
    ));
    assert!(core::ptr::eq(first_read.binding().file(), &first));
    assert!(core::ptr::eq(second_read.binding().file(), &second));
}
