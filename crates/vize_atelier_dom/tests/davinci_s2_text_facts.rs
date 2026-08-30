//! P2-11 text-fact id witnesses for component slot children.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "slot_element_compound_with_static_tail",
        r#"<Foo><strong>{{ points }} points</strong></Foo>"#,
    ),
    (
        "once_static_slot_sibling_before_compound_text",
        r#"<Foo v-once><i>x</i><span>{{ points }} points</span></Foo>"#,
    ),
    (
        "conditional_slot_template_compound_text",
        r#"<Foo><template v-if="layout === 'grid' || position === 'inside'" #files-top="{ files }"><div v-if="files?.length"><p>Files ({{ files?.length }})</p></div></template></Foo>"#,
    ),
    (
        "slot_element_compound_with_quoted_interp",
        r#"<Foo><div v-if="searchValue.length"><span>Add "{{ searchValue }}" as a new Client</span></div></Foo>"#,
    ),
    (
        "component_slot_root_compound_text",
        r#"<VbCard><va-hover #default="{ hover }"><div>slot - {{ hover }}</div></va-hover></VbCard>"#,
    ),
];

#[test]
fn s2_text_fact_ids_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
