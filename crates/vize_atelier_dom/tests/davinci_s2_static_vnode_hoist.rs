//! Static child vnode hoists, compared byte-for-byte.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "nested_select_static_option_before_for",
        r#"<div><select v-model="msg"><option value=""> Select </option><option v-for="item in items" :value="item">{{ item }}</option></select></div>"#,
    ),
    (
        "v_if_static_child_and_text",
        r#"<div v-if="ok"><span></span> x</div>"#,
    ),
    (
        "v_if_static_child_with_attrs",
        r#"<div v-if="ok"><span class="x">hello</span></div>"#,
    ),
    (
        "nested_v_show_static_child",
        r#"<div><span v-show="ok"><b>Downloading update</b></span></div>"#,
    ),
    (
        "airi_titlebar_dynamic_native_ancestor_hoists_static_grandchild",
        r#"<div :class="root"><div flex drag-region><button @click="run"><span>{{ title }}</span></button><div w-full drag-region></div></div></div>"#,
    ),
    (
        "airi_screen_capture_component_slot_hoists_static_break",
        r#"<DialogDescription>{{ summary }}<ol><li>{{ step }}<br><span>{{ note }}</span></li></ol></DialogDescription>"#,
    ),
    (
        "v_for_item_root_hoists_static_child_vnodes",
        r#"<div v-for="item in items" :key="item.id"><span class="icon"></span>{{ item.name }}</div>"#,
    ),
];

#[test]
fn s2_static_child_vnode_hoists_match_the_shipped_dom_lane() {
    support::assert_s2_matches_shipped(BATTERY);
}
