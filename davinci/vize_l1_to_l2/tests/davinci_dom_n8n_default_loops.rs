//! Ordinary default loops must not also evaluate as dynamic slot descriptors.
#![expect(
    clippy::disallowed_types,
    reason = "complete differential receipts use std strings"
)]

mod davinci_dom_corpus_support;
use davinci_dom_corpus_support::{Lane, Report, compare_sfc_template_lane};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-slot-loop-regression/UserSelect.vue.txt"
);

#[test]
fn ordinary_default_loops_keep_whole_module_parity_with_named_slots() {
    let cases = [
        ("n8n_original_user_select", ORIGINAL),
        (
            "ordinary_component_default",
            r#"<template><Select><template v-if="prefix" #prefix>header</template><Option v-for="row in loadRows()" :key="row.id">{{ row.label }}</Option></Select></template>"#,
        ),
        (
            "ordinary_element_default",
            r#"<template><Select><template v-if="prefix" #prefix>header</template><span v-for="row in rows" :key="row.id">{{ row.label }}</span></Select></template>"#,
        ),
        (
            "loop_before_named_slot",
            r#"<template><Select><Option v-for="row in rows" :key="row.id">{{ row.label }}</Option><template v-if="footer" #footer>end</template></Select></template>"#,
        ),
        (
            "template_default_outlet",
            r#"<template><Select><template v-if="prefix" #prefix>header</template><template v-for="row in rows"><slot :row="row" /></template></Select></template>"#,
        ),
        (
            "template_default_element",
            r#"<template><Select><template v-if="prefix" #prefix>header</template><template v-for="row in rows"><span :key="row.id">{{ row.label }}</span></template></Select></template>"#,
        ),
        (
            "named_loop_stays_dynamic",
            r#"<template><Select><template v-for="row in rows" #[row.id]>{{ row.label }}</template></Select></template>"#,
        ),
        (
            "mixed_actual_and_default_loops",
            r#"<template><Select><template v-for="name in names" #[name]>named</template><Option v-for="row in rows" :key="row.id">{{ row.label }}</Option></Select></template>"#,
        ),
    ];
    for lane in [Lane::Default, Lane::Prefixed, Lane::Bindings] {
        let mut report = Report::default();
        for (name, source) in cases {
            compare_sfc_template_lane(name, source, &mut report, lane);
        }
        assert_eq!(report.files, 8);
        assert_eq!(report.templates, 8);
        assert_eq!(report.compared, 8);
        assert_eq!(report.old_error_skips, 0);
        assert_eq!(report.s2_refusal_count, 0, "{:?}", report.s2_refusals);
        assert_eq!(report.divergence_count, 0, "{:?}", report.divergences);
        assert_eq!(report.s2_refusals, Vec::<String>::new());
        assert_eq!(report.divergences, Vec::<String>::new());
    }
}
