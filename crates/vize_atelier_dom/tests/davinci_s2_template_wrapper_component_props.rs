//! Reduced template-wrapper component-prop parity witnesses.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "scoped_slot_static_component_prop_hoists",
        r#"<Tree><template #title="{ key, title }"><Dropdown><template #overlay><Menu><MenuItem key="1">one</MenuItem><MenuItem key="2">two</MenuItem></Menu></template></Dropdown></template></Tree>"#,
    ),
    (
        "template_for_component_static_bind_prop_hoists",
        r#"<Row><template v-for="component in group.children" :key="component.title"><Col :xs="24" :sm="12" :lg="8" :xl="6"><Card /></Col></template></Row>"#,
    ),
    (
        "template_for_if_component_uses_branch_key_without_duplicate_authored_key",
        r#"<Menu><template v-for="m in menus"><template v-if="m.children"><MenuGroup :key="m.order" :title="m.title" /></template><template v-else><MenuItem :key="m.path" /></template></template></Menu>"#,
    ),
    (
        "scoped_slot_dynamic_param_prop_stays_inline",
        r#"<Tree><template #title="{ title }"><MenuItem :title="title" /></template></Tree>"#,
    ),
];

#[test]
fn s2_template_wrapper_component_props_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
