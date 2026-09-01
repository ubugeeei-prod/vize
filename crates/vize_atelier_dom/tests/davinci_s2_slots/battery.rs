use super::*;

const BATTERY: &[(&str, &str)] = &[
    (
        "named_header_text",
        "<Foo><template #header>title</template></Foo>",
    ),
    (
        "named_header_interp",
        "<Foo><template #header>hello {{ msg }}</template></Foo>",
    ),
    (
        "named_then_default",
        "<Foo><template #header>title</template>hello</Foo>",
    ),
    ("bare_template_default", "<Foo><template>x</template></Foo>"),
    (
        "default_then_named",
        "<Foo>hello<template #header>title</template></Foo>",
    ),
    (
        "named_header_span",
        "<Foo><template #header><span></span></template></Foo>",
    ),
    (
        "implicit_default_svg",
        "<Foo><svg><path d=\"M0 0\" /></svg></Foo>",
    ),
    (
        "implicit_default_mathml",
        "<Foo><math><mi>x</mi></math></Foo>",
    ),
    (
        "named_header_extra_attr",
        r#"<Foo><template #header id="x">x</template></Foo>"#,
    ),
    (
        "named_header_v_once",
        r#"<Foo><template #header v-once>x</template></Foo>"#,
    ),
    (
        "named_header_v_memo",
        r#"<Foo><template #header v-memo="[ok]">x</template></Foo>"#,
    ),
    (
        "hyphenated_slot",
        "<Foo><template #foo-bar>x</template></Foo>",
    ),
    (
        "two_named",
        "<Foo><template #header>title</template><template #footer>end</template></Foo>",
    ),
    ("empty_named", "<Foo><template #header></template></Foo>"),
    ("ws_named", "<Foo><template #header>  </template></Foo>"),
    ("component_v_slot_header", "<Foo v-slot:header>title</Foo>"),
    ("component_v_slot", "<Foo v-slot>title</Foo>"),
    (
        "component_v_slot_empty_params",
        r#"<Foo v-slot="">title</Foo>"#,
    ),
    ("component_hash_header", "<Foo #header>title</Foo>"),
    (
        "dynamic_slot_name",
        r#"<Foo><template #[name]>x</template></Foo>"#,
    ),
    (
        "named_slot_empty_params",
        r#"<Foo><template #header="">x</template></Foo>"#,
    ),
    (
        "scoped_ident",
        r#"<Foo><template #header="p">x</template></Foo>"#,
    ),
    (
        "scoped_destructure",
        r#"<Foo><template #header="{ foo }">x</template></Foo>"#,
    ),
    (
        "create_slots_if",
        r#"<Foo><template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_if_v_once",
        r#"<Foo><template #header v-if="ok" v-once>x</template></Foo>"#,
    ),
    (
        "create_slots_if_v_memo",
        r#"<Foo><template #header v-if="ok" v-memo="[ok]">x</template></Foo>"#,
    ),
    (
        "create_slots_empty_params",
        r#"<Foo><template #header="" v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_if_extra_attr",
        r#"<Foo><template #header id="x" v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_for",
        r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#,
    ),
    (
        "create_slots_for_v_once",
        r#"<Foo><template v-for="i in n" #header v-once>x</template></Foo>"#,
    ),
    (
        "create_slots_for_v_memo",
        r#"<Foo><template v-for="i in n" #header v-memo="[i]">x</template></Foo>"#,
    ),
    (
        "create_slots_for_extra_attr",
        r#"<Foo><template v-for="i in n" #header id="x">x</template></Foo>"#,
    ),
    (
        "create_slots_if_and_static",
        r#"<Foo><template #header v-if="ok">x</template><template #footer>end</template></Foo>"#,
    ),
    (
        "create_slots_default_and_if",
        r#"<Foo>hello<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_if_else",
        r#"<Foo><template #header v-if="a">x</template><template #header v-else>y</template></Foo>"#,
    ),
    (
        "create_slots_if_else_if",
        r#"<Foo><template #header v-if="a">x</template><template #header v-else-if="b">y</template></Foo>"#,
    ),
    (
        "create_slots_if_span",
        r#"<Foo><template #header v-if="ok"><span></span></template></Foo>"#,
    ),
    (
        "create_slots_keeps_non_slot_if_placeholder_before_named_template",
        r#"<Foo><Icon v-if="showIcon" /><span v-else>{{ label }}</span><template v-if="$slots.menu" #menu><slot name="menu" /></template><template #append><slot name="append" /></template></Foo>"#,
    ),
    (
        "create_slots_keeps_non_slot_if_placeholder_before_scoped_forwarder",
        r#"<Foo><div v-if="$slots.viewer"><slot name="viewer" /></div><template v-if="$slots.progress" #progress="progress"><slot name="progress" v-bind="progress" /></template><template #toolbar="toolbar"><slot name="toolbar" v-bind="toolbar" /></template></Foo>"#,
    ),
    (
        "nested_slot_forwarder_prefers_render_slot_before_create_vnode",
        r#"<Foo><template #aside><Copy><template v-if="$slots.copy" #copy><slot name="copy" /></template><template #backdrop><Backdrop /></template></Copy></template></Foo>"#,
    ),
    (
        "create_slots_condition_padding",
        r#"<Foo><template #header v-if="
  ok &&
  ready
">x</template></Foo>"#,
    ),
    (
        "create_slots_scoped_param_padding",
        r#"<Foo><template #header="slotProps " v-if="ok"><slot v-bind="slotProps" /></template></Foo>"#,
    ),
    (
        "create_slots_scoped",
        r#"<Foo><template #header="p" v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_dynamic_name",
        r#"<Foo><template #[name] v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_hyphenated",
        r#"<Foo><template #foo-bar v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_empty",
        r#"<Foo><template #header v-if="ok"></template></Foo>"#,
    ),
    (
        "create_slots_for_aliases",
        r#"<Foo><template v-for="(v, k, i) in n" #header>x</template></Foo>"#,
    ),
    (
        "create_slots_for_keyed_dynamic_forwarded_outlet",
        r#"<Foo><template v-for="(_, name) in $slots" :key="name" #[name]="slotData"><slot :name="name" v-bind="slotData || {}" /></template></Foo>"#,
    ),
    (
        "create_slots_default_interp",
        r#"<Foo>hello {{ msg }}<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "create_slots_nested_template",
        r#"<Foo><template #header v-if="ok"><template #inner>x</template></template></Foo>"#,
    ),
    (
        "create_slots_nested_template_interp",
        r#"<Foo><template v-for="i in n" #header><template #inner>{{ i }}</template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template",
        r#"<Foo><template #header><template #inner>x</template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template_interp",
        r#"<Foo><template #header><template #inner>{{ msg }}</template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template_multiple",
        r#"<Foo><template #header><template #inner><b></b><i></i></template></template></Foo>"#,
    ),
    (
        "nested_named_slot_template_empty",
        r#"<Foo><template #header><template #inner></template></template></Foo>"#,
    ),
    (
        "stray_named_slot_template_inside_native",
        r#"<div><template #inner>x</template></div>"#,
    ),
    (
        "stray_named_slot_template_interp",
        r#"<div><template #inner>{{ msg }}</template></div>"#,
    ),
    (
        "stray_named_slot_template_multiple",
        r#"<div><template #inner><b></b><i></i></template></div>"#,
    ),
    (
        "stray_named_slot_template_empty",
        r#"<div><template #inner></template></div>"#,
    ),
    (
        "slot_outlet_fallback_stray_template",
        r#"<slot><template #inner>{{ msg }}</template></slot>"#,
    ),
    (
        "unwrapped_if_nested_slot_keeps_siblings",
        r#"<Foo><template v-if="ok"><span>x</span><template #header>y</template></template></Foo>"#,
    ),
    (
        "unwrapped_for_nested_slot_keeps_siblings",
        r#"<Foo><template v-for="i in n"><span>x</span><template #header>y</template></template></Foo>"#,
    ),
    (
        "unwrapped_if_single_nested_slot_stays_default",
        r#"<Foo><template v-if="ok"><template #header>y</template></template></Foo>"#,
    ),
    (
        "unwrapped_for_single_nested_slot_stays_default",
        r#"<Foo><template v-for="i in n"><template #header>y</template></template></Foo>"#,
    ),
    (
        "unwrapped_if_two_nested_slots_keeps_both",
        r#"<Foo><template v-if="ok"><template #header>h</template><template #footer>f</template></template></Foo>"#,
    ),
    (
        "unwrapped_for_two_nested_slots_keeps_both",
        r#"<Foo><template v-for="i in n"><template #header>h</template><template #footer>f</template></template></Foo>"#,
    ),
    (
        "component_slot_params_preserve_authored_padding",
        r#"<Foo v-slot="slotProps "><slot v-bind="slotProps" /></Foo>"#,
    ),
    (
        "slotted_component_legacy_patchless_concat_prop",
        r#"<Foo><Bar :foo="'a' + i + 'b'"><Baz /></Bar></Foo>"#,
    ),
    (
        "slotted_for_component_legacy_patchless_concat_prop",
        r#"<Foo><Bar v-for="(item, index) in items" :key="item.key" :label="'Label' + index" :prop="'items.' + index + '.value'" :rules="{ required: true }"><Baz /></Bar></Foo>"#,
    ),
    (
        "render_slot_if_authored_key_dedupes_props",
        r#"<template v-for="(item, index) in items"><slot v-if="index < items.length - 1" name="separator" :key="'separator-' + index" /></template>"#,
    ),
    (
        "transition_forwarded_slot_hoists_static_props",
        r#"<Transition name="fade"><slot /></Transition>"#,
    ),
    (
        "transition_group_slot_hoists_static_props",
        r#"<TransitionGroup name="fade"><div v-for="item in items" :key="item.id">{{ item.label }}</div></TransitionGroup>"#,
    ),
    (
        "component_dynamic_static_props_with_complex_slot_stays_inline",
        r#"<Foo :value-on-clear="null" :empty-values="[undefined, null]"><div><Bar /></div></Foo>"#,
    ),
    (
        "dynamic_slot_name_hole",
        r#"<Foo><template #[]>x</template></Foo>"#,
    ),
];

#[test]
fn s2_named_slots_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
