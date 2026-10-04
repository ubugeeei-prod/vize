use vize_l0::Allocator;
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::lower_selected_setup_sfc_native;
use vize_l2::{file::DeclarationKind, resolution::Usage};
use vize_l3::decision::ssr::{
    SsrPart, SsrSetupReadKind, SsrUnsupported, build_native_selected_setup_ssr_decisions,
};

#[test]
fn original_setup_reads_retain_actual_occurrences_bindings_and_distinct_scopes() {
    for (source, expected) in [
        (
            "<script setup>const value=42</script><template>{{value}}</template>",
            SsrSetupReadKind::SetupConst,
        ),
        (
            "<script setup lang='ts'>let value:string='雪'</script><template>{{value}}</template>",
            SsrSetupReadKind::SetupLet,
        ),
        (
            "<script setup>var 雪=42</script><template>{{雪}}<i/></template>",
            SsrSetupReadKind::SetupLet,
        ),
    ] {
        let arena = Allocator::default();
        let observation =
            lower_selected_setup_sfc_native(&arena, source, DescriptorOptions::default());
        let admitted = observation.admitted().unwrap();
        let setup = admitted.setup();
        let file = setup.file();
        let provenance = file.artifact().provenance().to_vec();
        let analysis = build_native_selected_setup_ssr_decisions(setup).unwrap();
        assert!(core::ptr::eq(analysis.setup(), setup));
        assert!(core::ptr::eq(analysis.file(), file));
        assert!(core::ptr::eq(
            analysis.owner(),
            observation.original().template().unwrap()
        ));
        assert!(core::ptr::eq(analysis.artifact().source(), source));
        let facts = analysis.ssr().unwrap();
        assert!(facts.unsupported().is_empty());
        let interpolations: Vec<_> = facts
            .parts()
            .iter()
            .filter_map(|part| {
                let SsrPart::Interpolation {
                    node,
                    interpolation,
                } = part
                else {
                    return None;
                };
                Some((*node, interpolation))
            })
            .collect();
        let [(node, interpolation)] = interpolations.as_slice() else {
            panic!("one root read")
        };
        let original = file.native_interpolation(*node).unwrap();
        assert_eq!(original.input().operand().full_span(), interpolation.span);
        let expression = analysis.expression(*node).unwrap();
        let resolution = expression.resolution();
        assert!(core::ptr::eq(resolution.file(), file));
        assert_ne!(resolution.scope(), Some(setup.scope()));
        let table = resolution.table().unwrap();
        let [occurrence] = table.occurrences() else {
            panic!("actual identifier occurrence")
        };
        let [read] = expression.reads() else {
            panic!("actual immutable read")
        };
        assert!(core::ptr::eq(read.occurrence(), occurrence));
        assert_eq!(occurrence.usage, Usage::Read);
        assert_eq!(read.kind(), expected);
        let binding = read.binding();
        assert!(resolution.accepts(binding));
        assert!(setup.binding(binding).is_ok());
        let declaration = binding.declaration().unwrap();
        assert_eq!(declaration.unit, setup.unit());
        assert_eq!(declaration.scope, setup.scope());
        assert_eq!(declaration.name.as_str(), occurrence.name);
        let span = table.expression().authored_span(occurrence.span).unwrap();
        assert_eq!(span.slice(source), occurrence.name);
        assert_eq!(
            expected == SsrSetupReadKind::SetupConst,
            declaration.kind == DeclarationKind::Const
        );
        assert_eq!(file.artifact().provenance(), provenance);
        assert_eq!(
            analysis.tables().nodes.len(),
            file.artifact().node_count() as usize
        );
    }
}

#[test]
fn original_primitive_literals_have_complete_same_file_resolution_without_reads() {
    for value in ["true", "null", "42", "123n", "'雪 & <one>'"] {
        let arena = Allocator::default();
        let source =
            format!("<script setup>const unused=0</script><template>{{{{{value}}}}}</template>");
        let observation =
            lower_selected_setup_sfc_native(&arena, &source, DescriptorOptions::default());
        let admitted = observation.admitted().unwrap();
        let analysis = build_native_selected_setup_ssr_decisions(admitted.setup()).unwrap();
        let facts = analysis.ssr().unwrap();
        assert!(facts.unsupported().is_empty());
        let [SsrPart::Interpolation { node, .. }] = facts.parts() else {
            panic!("one literal")
        };
        let expression = analysis.expression(*node).unwrap();
        assert!(expression.reads().is_empty());
        assert!(
            expression
                .resolution()
                .table()
                .unwrap()
                .occurrences()
                .is_empty()
        );
        assert!(core::ptr::eq(
            expression.resolution().file(),
            admitted.setup().file()
        ));
    }
}

#[test]
fn complete_original_expressions_keep_the_exact_bounded_target_refusal() {
    for (template, expected) in [
        ("{{value+1}}", SsrUnsupported::Expression),
        ("{{value.length}}", SsrUnsupported::Expression),
        ("hello {{value}}!", SsrUnsupported::RootTextGrouping),
        ("{{value}}{{true}}", SsrUnsupported::RootTextGrouping),
        ("{{value}}<i/>{{value}}", SsrUnsupported::Operation),
    ] {
        let arena = Allocator::default();
        let source =
            format!("<script setup>const value='ab'</script><template>{template}</template>");
        let observation =
            lower_selected_setup_sfc_native(&arena, &source, DescriptorOptions::default());
        let admitted = observation.admitted().unwrap();
        assert!(admitted.setup().file().is_complete());
        let analysis = build_native_selected_setup_ssr_decisions(admitted.setup()).unwrap();
        let rejected = &analysis.ssr().unwrap().unsupported()[0];
        assert_eq!(rejected.reason, expected);
        assert!(
            source
                .get(rejected.span.start as usize..rejected.span.end as usize)
                .is_some()
        );
    }
}
