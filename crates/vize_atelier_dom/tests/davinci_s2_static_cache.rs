//! P2-11 static-cache witness: when a root hoist enables the legacy
//! `_cache` gate, sibling static element children are cached as one array.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "hoisted_root_caches_static_children_array",
        r#"<div class="root"><span>a</span><span>b</span></div>"#,
    ),
    (
        "cached_static_child_keeps_nested_element_array_shape",
        r#"<div class="root"><button><span>x</span></button></div>"#,
    ),
    (
        "cached_static_child_before_dynamic_sibling",
        r#"<div class="root"><button><span>x</span></button><i :id="foo"></i></div>"#,
    ),
    (
        "static_slot_hoist_keeps_inline_space_text_child",
        r#"<Variant><div><span>Project</span> <span>AIRI</span></div></Variant>"#,
    ),
    (
        "airi_loading_component_formats_nested_static_props_inside_for",
        r#"<div flex flex-col gap-2><div v-for="file in files" :key="file.filename" max-w-full flex flex-col gap="1 sm:2"><div grid="~ cols-[85%_15%]" justify-between text="xs sm:sm neutral-600 dark:neutral-400"><div flex items-center gap-1>{{ file.filename }}</div></div></div></div>"#,
    ),
    (
        "airi_loading_modules_formats_static_slot_child_props_inside_for",
        r#"<ul><li v-for="[moduleName, module] in resources" :key="moduleName"><WindowRouterLink :to="`/settings/modules/${moduleName}`" label="settings"><div flex items-center gap-1><div flex items-center gap-1><span>{{ moduleName }}</span></div></div></WindowRouterLink></li></ul>"#,
    ),
    (
        "svg_static_bind_child_cache_formats_props_like_the_shipped_snapshot",
        r#"<svg viewBox="0 0 400 400"><circle :r="150" cx="200" cy="200" :stroke="'darkgrey'" :stroke-width="40" fill="none" /><circle :stroke-dasharray="progress" /></svg>"#,
    ),
];

#[test]
fn s2_static_child_cache_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
