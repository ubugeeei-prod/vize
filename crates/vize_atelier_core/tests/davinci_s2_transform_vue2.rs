//! P2-9 Vue 2 atelier comparator: the installment-7 sugar dual-runs
//! against the shipped legacy lane under dialect V2.
//!
//! Compiled only with `--features legacy` (`[[test]] required-features`):
//! the old lane's `.sync` / `slot-scope` / filter expansion is behind
//! that feature, and a Vue 2 dual-run without it would compare S2's
//! rewrite against an inert Vue 3 tree. The default workspace suite
//! still pins the Vue 3 battery; clippy-and-test invokes this target
//! explicitly so a wiring regression fails in CI.
//!
//! Interpolation filters are in this battery (installment 9): a lone
//! `{{ msg | cap }}` wrap-equals `_filter_cap(msg)`; a mixed run that
//! absorbed a pipe into a compound opaque compares the authored pipe
//! exactly. Neither is silently skipped.

#![cfg(feature = "legacy")]
#![allow(clippy::disallowed_types, clippy::disallowed_macros)]

mod s2_support;

use s2_support::{
    Counters, HoistCounters, SlotCounters, SurfaceCounters, TextCounters, compare_with,
};
use vize_carton::config::VueVersion;

/// Vue 2 sugar the two lanes both claim to legalize, plus one
/// sugar-free template so the rest of the projection still agrees
/// under `walks=7`.
const BATTERY: &[(&str, &str)] = &[
    ("sync", r#"<Comp :title.sync="heading"/>"#),
    ("sync-camel", r#"<Comp :title.sync.camel="heading"/>"#),
    (
        "slot-scope",
        r#"<Comp><template slot-scope="props">x</template></Comp>"#,
    ),
    (
        "named-slot-scope",
        r#"<MyComp><template slot="header" slot-scope="props">{{ props.title }}</template></MyComp>"#,
    ),
    ("if", r#"<div v-if="ok">x</div>"#),
    (
        "native-keycode",
        r#"<Comp @click.native @keyup.13="onKey"/>"#,
    ),
    ("filter", "{{ msg | cap }}"),
    ("filter-mixed", "hello {{ msg | cap }}"),
    ("filter-args", "{{ a | f(b) }}"),
];

fn expected() -> Counters {
    Counters {
        templates_seen: 9,
        compared: 9,
        skipped_legacy_flag: 0,
        skipped_old_parse_errors: 0,
        skipped_s2_errors: 0,
        if_ops: 1,
        branches: 1,
        keys_static: 0,
        keys_dynamic: 0,
        keys_wrapper: 0,
        keys_dynamic_arg: 0,
        keys_compound: 0,
        conditions_compound: 0,
        for_ops: 0,
        for_values: 0,
        for_keys: 0,
        for_indexes: 0,
        for_values_absent: 0,
        for_compound: 0,
        slots: SlotCounters {
            units: 2,
            groups: 2,
            group_params: 2,
            groups_invented: 1,
            groups_dynamic: 0,
            units_conditional: 0,
            units_forwarded: 0,
            units_filler_default: 0,
            outlets: 0,
            outlets_dynamic: 0,
        },
        text: TextCounters {
            units: 6,
            parts_static: 3,
            parts_dynamic: 4,
            compound_units: 1,
            vpre_templates: 0,
            entity_templates: 0,
            rawtext_excluded: 0,
            parts_compound: 0,
            parts_filter: 2,
        },
        surfaces: SurfaceCounters {
            owners: 8,
            attrs: 0,
            binds: 0,
            binds_dynamic: 0,
            binds_spread: 0,
            ons: 2,
            ons_dynamic: 0,
            ons_spread: 0,
            directives: 0,
            models: 2,
            models_invalid: 0,
            models_dynamic_arg: 0,
            models_pattern_scope: 0,
            keys_excluded: 0,
            builtins_excluded: 0,
            wrapper_attrs: 0,
            entity_templates: 0,
            table_templates: 0,
            values_compound: 0,
        },
        hoist: HoistCounters {
            elements: 7,
            whole: 0,
            props: 0,
            wrapper_hoists: 0,
            comments_elements: 0,
            builtins_subtrees: 0,
            consts_templates: 0,
            classifier_templates: 0,
            models_templates: 0,
            tree_templates: 0,
            vpre_templates: 0,
            table_templates: 0,
        },
    }
}

#[test]
fn the_vue2_battery_dual_runs_with_pinned_counts() {
    let mut counters = Counters::default();
    for (name, source) in BATTERY {
        compare_with(name, source, &mut counters, VueVersion::V2);
    }
    assert_eq!(
        counters,
        expected(),
        "the Vue 2 differential battery accounting moved: re-pin this witness"
    );
}
