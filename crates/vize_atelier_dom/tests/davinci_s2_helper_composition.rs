//! P2-11 helper ordering and component-slot composition witnesses.
//!
//! These are reduced from the hydrated DOM corpus divergences where
//! component slots combine text helpers, conditional comment helpers,
//! and slot wrappers in one helper preamble.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "component_text_then_if_span",
        r#"<Foo>hello<span v-if="ok">x</span></Foo>"#,
    ),
    (
        "component_if_span_then_text",
        r#"<Foo><span v-if="ok">x</span>hello</Foo>"#,
    ),
    (
        "component_template_if_text_then_text",
        r#"<Foo><template v-if="ok">x</template>hello</Foo>"#,
    ),
    (
        "component_text_then_template_if_text",
        r#"<Foo>hello<template v-if="ok">x</template></Foo>"#,
    ),
    (
        "component_text_then_conditional_named_slot",
        r#"<Foo>hello<template #header v-if="ok">x</template></Foo>"#,
    ),
    (
        "component_conditional_named_slot_then_text",
        r#"<Foo><template #header v-if="ok">x</template>hello</Foo>"#,
    ),
    (
        "component_nested_text_then_if_component",
        r#"<Foo>hello<Bar v-if="ok">x</Bar></Foo>"#,
    ),
    (
        "component_if_component_then_text",
        r#"<Foo><Bar v-if="ok">x</Bar>hello</Foo>"#,
    ),
    (
        "text_slot_before_v_if_fallback",
        r#"<Foo>hello<template v-if="ok"><span>x</span></template></Foo>"#,
    ),
    (
        "v_show_sibling_before_v_for",
        r#"<section><div v-show="visible"></div><p v-for="item in items">{{ item }}</p></section>"#,
    ),
    (
        "v_show_component_slot",
        r#"<Dialog v-show="open"><span>Body</span></Dialog>"#,
    ),
    (
        "nested_trigger_icon_resolves_in_child_before_parent_order",
        r#"<DropdownMenu><DropdownMenuTrigger><Icon /></DropdownMenuTrigger></DropdownMenu>"#,
    ),
    (
        "conditional_component_text_slot_imports_text_before_comment",
        r#"<Button v-if="primaryActionLabel">{{ primaryActionLabel }}</Button>"#,
    ),
    (
        "slot_outlet_before_conditional_component_text_slots",
        r#"<div><slot /><Button v-if="primaryActionLabel">{{ primaryActionLabel }}</Button><Button v-if="secondaryActionLabel">{{ secondaryActionLabel }}</Button></div>"#,
    ),
    (
        "element_if_before_conditional_component_text_slots",
        r#"<section><p v-if="detail">{{ detail }}</p><div v-if="primaryActionLabel || secondaryActionLabel || $slots.default"><slot /><Button v-if="primaryActionLabel">{{ primaryActionLabel }}</Button><Button v-if="secondaryActionLabel">{{ secondaryActionLabel }}</Button></div></section>"#,
    ),
    (
        "static_text_before_if_imports_text_before_comment",
        r#"<section><p>AIRI account</p><p v-if="detail">{{ detail }}</p><div v-if="primaryActionLabel || secondaryActionLabel || $slots.default"><slot /><Button v-if="primaryActionLabel">{{ primaryActionLabel }}</Button><Button v-if="secondaryActionLabel">{{ secondaryActionLabel }}</Button></div></section>"#,
    ),
    (
        "compound_text_before_if_imports_text_before_comment",
        r#"<div><span>{{ branch }}@{{ commit }}</span><Button v-if="ok">{{ label }}</Button></div>"#,
    ),
    (
        "slot_fallback_component_resolves_before_owner_component",
        r#"<DropdownMenuTrigger><slot name="trigger"><span>{{ label }}</span><Icon icon="x" /></slot></DropdownMenuTrigger><DropdownMenuItem><span>Item</span></DropdownMenuItem>"#,
    ),
    (
        "slot_outlet_and_static_child_keeps_root_element_vnode_before_slot",
        r#"<div><slot /><span>{{ label }}</span></div>"#,
    ),
    (
        "template_slot_carrier_imports_slot_before_element_vnode",
        r#"<Collapsible :default="expand"><template #trigger="slotProps"><button :class="['w-full']"><slot name="title"><div>{{ title }}</div></slot></button></template><div :class="innerClass"><slot /></div></Collapsible>"#,
    ),
    (
        "nested_template_slot_carrier_before_later_slot_outlet",
        r#"<Section><FieldRange><template #label><div>{{ label }}</div></template></FieldRange><label><div><slot name="label">{{ fallback }}</slot></div><SelectTab /></label></Section>"#,
    ),
    (
        "component_default_element_wraps_slot_outlet",
        r#"<Section><label><div><slot name="label">{{ fallback }}</slot></div><SelectTab /></label></Section>"#,
    ),
    (
        "earlier_root_component_template_slot_before_later_root_slot_outlet",
        r#"<Section><FieldRange><template #label><div>{{ label }}</div></template></FieldRange></Section><Section><label><div><slot name="label">{{ fallback }}</slot></div><SelectTab /></label></Section>"#,
    ),
    (
        "explicit_slot_template_nested_slot_prefers_render_slot_helper",
        r#"<Carousel><template #slides><SwiperSlide v-for="item in items"><div :class="cls"></div><p class="x"><slot name="referenceText" /></p></SwiperSlide></template></Carousel>"#,
    ),
    (
        "component_v_slot_direct_slot_prefers_render_slot_helper",
        r#"<MenuItem v-slot="{ active }" v-bind="$attrs"><a :class="[active ? 'on' : 'off']"><slot :active="active" /></a></MenuItem>"#,
    ),
    (
        "component_v_slot_nested_slot_prefers_render_slot_helper",
        r#"<Listbox v-slot="{ open }"><ScalarFloating><ListboxButton><slot :open="open" /></ListboxButton><template #floating="{ width }"><div v-if="open" :style="{ width }"><Item v-for="option in options" :key="option.id" /></div></template></ScalarFloating></Listbox>"#,
    ),
    (
        "component_v_slot_carrier_before_later_slot_outlet_prefers_render_slot",
        r#"<DefineMonthTemplate v-slot="{ date }"><div>{{ date }}</div></DefineMonthTemplate><CalendarRoot v-slot="{ grid }"><CalendarPrevButton><slot name="calendar-prev-icon" /></CalendarPrevButton></CalendarRoot>"#,
    ),
    (
        "dynamic_bind_key_prefers_normalize_props_before_class",
        r#"<component :is="self ? 'MkA' : 'a'" :[attr]="maybeRelativeUrl" :class="$style.root"><slot /></component>"#,
    ),
    (
        "implicit_slot_outlet_before_named_slot_vnode_prefers_render_slot_helper",
        r#"<CommonTooltip><VDropdown><slot /><template #popper><div ref="el"></div></template></VDropdown></CommonTooltip>"#,
    ),
    (
        "dynamic_component_before_directive_component_does_not_prefer_resolve_component",
        r#"<div><component :is="icon" /><Foo v-tippy="tip" /></div>"#,
    ),
    (
        "conditional_named_slot_direct_v_for_prefers_render_list_before_create_slots",
        r#"<ChatPrompt><template v-if="attachments.length" #header><Button v-for="(file, index) in attachments" :key="index" :label="file.name" /></template><template #body><Editor /></template></ChatPrompt>"#,
    ),
    (
        "source_order_default_before_named_slot_keeps_helper_order",
        r#"<Page><Button><template #right><Icon v-if="ok" /></template>{{ label }}</Button><template #notice><Icon /> {{ notice }}</template></Page>"#,
    ),
    (
        "source_order_default_if_before_late_named_slot_keeps_helper_order",
        r#"<PublicView><form><Notice v-if="done">{{ ok }}</Notice><Notice v-if="error">{{ error }}</Notice></form><template #notice><Icon /> {{ msg }}</template></PublicView>"#,
    ),
    (
        "footer_transition_prefers_builtin_before_default_transition_group",
        r#"<PageWithHeader><div><TransitionGroup><template v-for="item in timeline" :key="item.id"><Message :message="item" /></template></TransitionGroup></div><template #footer><Transition name="fade"><div v-show="showIndicator"></div></Transition></template></PageWithHeader>"#,
    ),
];

#[test]
fn s2_helper_composition_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
