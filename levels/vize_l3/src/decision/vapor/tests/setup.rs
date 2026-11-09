use super::options;
use crate::decision::vapor::{
    VaporPart, VaporUnsupported, VaporValueKind, build_native_selected_setup_vapor_decisions,
};
use crate::decision::{build_decisions, policy::TargetPolicy};
use vize_l0::Allocator;
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;

#[test]
fn original_setup_reads_borrow_the_same_complete_file_and_occurrences() {
    for (declaration, value, kind) in [
        ("const count=1", "count", VaporValueKind::SetupConst),
        ("let count=1", "count", VaporValueKind::SetupMutable),
        ("var 雪='🌸'", "雪", VaporValueKind::SetupMutable),
        ("const unused=1", "null", VaporValueKind::Scalar),
        ("const unused=1", "12n", VaporValueKind::Scalar),
        (r#"const count='a\nb'"#, "count", VaporValueKind::SetupConst),
        (
            r#"const count='a\ud83c\udf38b'"#,
            "count",
            VaporValueKind::SetupConst,
        ),
        (
            r#"const count='a\ufffdb'"#,
            "count",
            VaporValueKind::SetupConst,
        ),
    ] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup>{declaration}</script><template>{{{{{value}}}}}</template>"
        );
        let observation = lower_selected_setup_sfc_native(&arena, &source, options());
        assert_eq!(observation.original().issues(), [], "{source}");
        let selected = observation.admitted().unwrap();
        let setup = selected.setup();
        let analysis = build_native_selected_setup_vapor_decisions(setup).unwrap();
        assert!(core::ptr::eq(analysis.file(), setup.file()));
        assert!(core::ptr::eq(analysis.owner(), setup.owner()));
        assert_eq!(
            analysis.tables().nodes.len(),
            analysis.artifact().node_count() as usize
        );
        let facts = analysis.vapor().unwrap();
        assert_eq!(facts.unsupported(), []);
        let [
            VaporPart::Interpolation {
                node, expression, ..
            },
        ] = facts.parts(&facts.roots()[0]).unwrap()
        else {
            panic!("genuine original root interpolation")
        };
        assert_eq!(expression.kind(), kind);
        let row = expression.resolution();
        assert!(core::ptr::eq(row.file(), setup.file()));
        assert_eq!(row.node(), *node);
        assert!(setup.file().native_interpolation(*node).is_some());
        for occurrence in row.table().unwrap().occurrences() {
            assert!(
                setup
                    .binding(row.binding(occurrence.binding).unwrap())
                    .is_ok()
            );
        }
        // Neutral native analysis has no selected lexical runtime authority.
        let neutral = build_decisions(analysis.artifact(), TargetPolicy::Vapor).unwrap();
        assert_eq!(
            neutral.vapor().unwrap().unsupported()[0].reason,
            VaporUnsupported::Operation
        );
    }
}

#[test]
fn original_decoded_string_hazards_refuse_only_the_vapor_target() {
    for literal in [
        r#"'a\0b'"#,
        r#"'a\rb'"#,
        r#"'a\r\nb'"#,
        r#"'a\ud800b'"#,
        r#"'a\udc00b'"#,
    ] {
        for expression in ["count", literal] {
            let arena = Allocator::default();
            let source = alloc::format!(
                "<script setup>const count={literal}</script><template>{{{{{expression}}}}}</template>"
            );
            let observation = lower_selected_setup_sfc_native(&arena, &source, options());
            assert_eq!(observation.original().issues(), [], "{source}");
            let selected = observation.admitted().unwrap();
            assert!(selected.setup().file().is_complete());
            let analysis = build_native_selected_setup_vapor_decisions(selected.setup()).unwrap();
            assert_eq!(
                analysis.vapor().unwrap().unsupported()[0].reason,
                VaporUnsupported::StringNormalization
            );
        }
    }
}

#[test]
fn compound_and_escaped_original_reads_have_precise_target_refusals() {
    for expression in ["count+1", "\\u0063ount", "count.toString()"] {
        let arena = Allocator::default();
        let source = alloc::format!(
            "<script setup>let count=1</script><template>{{{{{expression}}}}}</template>"
        );
        let observation = lower_selected_setup_sfc_native(&arena, &source, options());
        assert_eq!(observation.original().issues(), [], "{source}");
        let selected = observation.admitted().unwrap();
        let analysis = build_native_selected_setup_vapor_decisions(selected.setup()).unwrap();
        assert_eq!(
            analysis.vapor().unwrap().unsupported()[0].reason,
            VaporUnsupported::Expression
        );
    }
}

#[test]
fn genuine_completed_control_handler_and_nested_inputs_keep_vapor_target_refusals() {
    for (template, reason) in [
        ("<div v-for='item in count'/>", VaporUnsupported::Operation),
        (
            "<div @click='var unused=$event;'/>",
            VaporUnsupported::Binding,
        ),
        (
            "<span>{{count}}</span>",
            VaporUnsupported::NestedInterpolation,
        ),
    ] {
        let arena = Allocator::default();
        let source =
            alloc::format!("<script setup>let count=1</script><template>{template}</template>");
        let observation = lower_selected_setup_sfc_native(&arena, &source, options());
        assert_eq!(observation.original().issues(), [], "{source}");
        let selected = observation.admitted().unwrap();
        assert!(selected.setup().file().is_complete());
        let analysis = build_native_selected_setup_vapor_decisions(selected.setup()).unwrap();
        assert_eq!(analysis.vapor().unwrap().unsupported()[0].reason, reason);
    }
}

#[test]
fn equal_foreign_root_ids_cannot_substitute_original_setup_facts() {
    let arena = Allocator::default();
    let source = "<script setup>let count=1</script><template>{{count}}</template>";
    let first = lower_selected_setup_sfc_native(&arena, source, options());
    let second = lower_selected_setup_sfc_native(&arena, source, options());
    let first_view = first.admitted().unwrap();
    let second_view = second.admitted().unwrap();
    let first_analysis = build_native_selected_setup_vapor_decisions(first_view.setup()).unwrap();
    let second_analysis = build_native_selected_setup_vapor_decisions(second_view.setup()).unwrap();
    let first_facts = first_analysis.vapor().unwrap();
    let second_facts = second_analysis.vapor().unwrap();
    assert_eq!(
        first_facts.roots()[0].node(),
        second_facts.roots()[0].node()
    );
    assert!(first_facts.parts(&second_facts.roots()[0]).is_none());
}
