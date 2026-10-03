//! Actual original bindings classify reads; lexical rows do not claim native custody.

use super::{Observed, bridge, expression};
use vize_l0::{Allocator, id::NodeId};
use vize_l1::embed::Lang;
use vize_l2::{
    file::{DeclarationKind, InitializerKind},
    resolution::Usage,
};
use vize_l3::decision::dom::{
    DomUnsupported, ValueKind,
    vue::{VueReadKind, build_vue_render_decisions},
};

#[test]
fn primitive_const_leaf_reads_keep_real_class_owner_and_constant_value() {
    let arena = Allocator::default();
    for initializer in ["1", "true", "null", "2n", "'雪'"] {
        let source = arena.alloc_str(&vize_l0::cstr!(
            "<script setup>const value={initializer};</script><template>{{{{value}}}}</template>"
        ));
        let observed = Observed::new(&arena, source).unwrap();
        let owner = expression(&arena, source, "value", Lang::Js).unwrap();
        let js = bridge(&arena, source, &owner).unwrap();
        let file = observed.file(&arena, &[js]).unwrap();
        let exposure = observed.exposure(&file).unwrap();
        let analysis = build_vue_render_decisions(&exposure).unwrap();
        assert!(analysis.dom().unwrap().unsupported().is_empty());
        let row = analysis.expression(NodeId::FIRST).unwrap();
        assert_eq!(row.value(), ValueKind::LiteralConstant);
        assert!(!row.value().is_dynamic());
        let [read] = row.reads() else {
            panic!("one genuine read")
        };
        assert_eq!(read.kind(), VueReadKind::SetupConst);
        assert_eq!(read.occurrence().usage, Usage::Read);
        assert!(core::ptr::eq(read.binding().file(), &file));
        let declaration = read.binding().declaration().unwrap();
        assert_eq!(declaration.kind, DeclarationKind::Const);
        assert_eq!(declaration.initializer, InitializerKind::PrimitiveLiteral);
        assert_eq!(declaration.script_unit(), Some(exposure.unit()));
        assert_eq!(declaration.scope, exposure.scope());
        let resolution = row.resolution();
        let table = resolution.table().unwrap();
        assert!(core::ptr::eq(read.occurrence(), &table.occurrences()[0]));
        assert!(core::ptr::eq(
            table.expression().ast,
            owner.expression().unwrap()
        ));
    }
}

#[test]
fn mutable_and_mixed_const_reads_keep_dynamic_semantics_and_duplicate_order() {
    let arena = Allocator::default();
    for (program, text, expected) in [
        ("let value=1;", "value", vec![VueReadKind::SetupLet]),
        ("var value=1;", "value", vec![VueReadKind::SetupLet]),
        (
            "const fixed=1;let value=2;",
            "fixed + value + fixed",
            vec![
                VueReadKind::SetupConst,
                VueReadKind::SetupLet,
                VueReadKind::SetupConst,
            ],
        ),
    ] {
        let source = arena.alloc_str(&vize_l0::cstr!(
            "<script setup>{program}</script><template>{{{{{text}}}}}</template>"
        ));
        let observed = Observed::new(&arena, source).unwrap();
        let owner = expression(&arena, source, text, Lang::Js).unwrap();
        let js = bridge(&arena, source, &owner).unwrap();
        let file = observed.file(&arena, &[js]).unwrap();
        let exposure = observed.exposure(&file).unwrap();
        let analysis = build_vue_render_decisions(&exposure).unwrap();
        let row = analysis.expression(NodeId::FIRST).unwrap();
        assert_eq!(row.value(), ValueKind::FileDependent);
        assert!(row.value().is_dynamic());
        assert_eq!(
            row.reads()
                .iter()
                .map(|read| read.kind())
                .collect::<Vec<_>>(),
            expected
        );
        for (read, occurrence) in row
            .reads()
            .iter()
            .zip(row.resolution().table().unwrap().occurrences())
        {
            assert!(core::ptr::eq(read.occurrence(), occurrence));
            assert_eq!(read.binding().id(), occurrence.binding);
            assert!(core::ptr::eq(read.binding().file(), &file));
        }
        if row.reads().len() == 3 {
            assert_eq!(row.reads()[0].binding().id(), row.reads()[2].binding().id());
            assert!(!core::ptr::eq(
                row.reads()[0].occurrence(),
                row.reads()[2].occurrence()
            ));
        }
    }
}

#[test]
fn immutable_reads_do_not_prove_calls_members_or_compound_expressions_constant() {
    let arena = Allocator::default();
    for text in [
        "value + value",
        "value.toString()",
        "value.length",
        "value.length = 2",
        "[value]",
        "({value})",
    ] {
        let source = arena.alloc_str(&vize_l0::cstr!(
            "<script setup>const value='x';</script><template>{{{{{text}}}}}</template>"
        ));
        let observed = Observed::new(&arena, source).unwrap();
        let owner = expression(&arena, source, text, Lang::Js).unwrap();
        let js = bridge(&arena, source, &owner).unwrap();
        let file = observed.file(&arena, &[js]).unwrap();
        let exposure = observed.exposure(&file).unwrap();
        let analysis = build_vue_render_decisions(&exposure).unwrap();
        let row = analysis.expression(NodeId::FIRST).unwrap();
        assert_eq!(row.value(), ValueKind::FileDependent, "{text}");
        assert!(
            row.reads()
                .iter()
                .all(|read| read.kind() == VueReadKind::SetupConst)
        );
        assert!(analysis.dom().unwrap().unsupported().is_empty());
    }
}

#[test]
fn unknown_const_initializer_never_publishes_an_immutable_or_partial_read_row() {
    let arena = Allocator::default();
    for initializer in ["original", "-1", "`text`"] {
        let source = arena.alloc_str(&vize_l0::cstr!(
            "<script setup>const original=1;const value={initializer};</script><template>{{{{original + value}}}}</template>"
        ));
        let observed = Observed::new(&arena, source).unwrap();
        let owner = expression(&arena, source, "original + value", Lang::Js).unwrap();
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
            [DomUnsupported::VueReadAccess]
        );
        assert_eq!(analysis.tables().nodes.len(), 1);
        assert_eq!(
            file.bindings()
                .last()
                .unwrap()
                .declaration()
                .unwrap()
                .initializer,
            InitializerKind::Unknown
        );
    }
}
