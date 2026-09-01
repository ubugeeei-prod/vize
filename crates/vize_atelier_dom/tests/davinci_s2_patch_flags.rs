//! P2-11 patch-flag witness for S2 DOM parity.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom_source;

struct Case {
    name: &'static str,
    src: &'static str,
    sites: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "text_child",
        src: "<div>{{ msg }}</div>",
        sites: &["1 /* TEXT */"],
    },
    Case {
        name: "class",
        src: r#"<div :class="cls"></div>"#,
        sites: &["2 /* CLASS */"],
    },
    Case {
        name: "component_class",
        src: r#"<Foo :class="cls" />"#,
        sites: &["8 /* PROPS */, [\"class\"]"],
    },
    Case {
        name: "dynamic_component_skips_is_prop",
        src: r#"<component :is="current" :active-class="klass" />"#,
        sites: &["8 /* PROPS */, [\"active-class\"]"],
    },
    Case {
        name: "dynamic_component_object_bind",
        src: r#"<component :is="current" v-bind="obj" />"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "style",
        src: r#"<div :style="style"></div>"#,
        sites: &["4 /* STYLE */"],
    },
    Case {
        name: "computed_key_style",
        src: r#"<div :style="{ [prop]: 'red' }"></div>"#,
        sites: &["4 /* STYLE */"],
    },
    Case {
        name: "computed_key_class",
        src: r#"<div :class="{ [prop]: true }"></div>"#,
        sites: &["2 /* CLASS */"],
    },
    Case {
        name: "id_and_computed_key_style",
        src: r#"<div :id="id" :style="{ [prop]: 'red' }"></div>"#,
        sites: &["12 /* STYLE, PROPS */, [\"id\"]"],
    },
    Case {
        name: "component_computed_key_style",
        src: r#"<Foo :style="{ [prop]: 'red' }" />"#,
        sites: &["8 /* PROPS */, [\"style\"]"],
    },
    Case {
        name: "computed_key_data",
        src: r#"<div :data="{ [k]: 1 }"></div>"#,
        sites: &["8 /* PROPS */, [\"data\"]"],
    },
    Case {
        name: "static_object_style_stays_patchless",
        src: r#"<div :style="{ color: 'red' }"></div>"#,
        sites: &[],
    },
    Case {
        name: "prop",
        src: r#"<div :id="id"></div>"#,
        sites: &["8 /* PROPS */, [\"id\"]"],
    },
    Case {
        name: "full_props",
        src: r#"<div :[key]="value"></div>"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "dynamic_event_full_props",
        src: r#"<div @[event]="handler"></div>"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "dynamic_event_text_full_props",
        src: r#"<div @[event]="handler">{{ msg }}</div>"#,
        sites: &["17 /* TEXT, FULL_PROPS */"],
    },
    Case {
        name: "full_props_need_hydration",
        src: r#"<div :[key].prop="value"></div>"#,
        sites: &["48 /* FULL_PROPS, NEED_HYDRATION */"],
    },
    Case {
        name: "prop_modifier_need_hydration",
        src: r#"<div :value.prop="value"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\".value\"]"],
    },
    Case {
        name: "click_event_props",
        src: r#"<div @click="handler"></div>"#,
        sites: &["8 /* PROPS */, [\"onClick\"]"],
    },
    Case {
        name: "click_stop_event_props",
        src: r#"<div @click.stop="handler"></div>"#,
        sites: &["8 /* PROPS */, [\"onClick\"]"],
    },
    Case {
        name: "click_prevent_stop_event_props",
        src: r#"<div @click.prevent.stop="handler"></div>"#,
        sites: &["8 /* PROPS */, [\"onClick\"]"],
    },
    Case {
        name: "click_once_capture_event_props",
        src: r#"<div @click.once.capture="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onClickOnceCapture\"]"],
    },
    Case {
        name: "click_right_event_props",
        src: r#"<div @click.right="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onContextmenu\"]"],
    },
    Case {
        name: "click_middle_event_props",
        src: r#"<div @click.middle="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onMouseup\"]"],
    },
    Case {
        name: "component_keyup_props",
        src: r#"<Foo @keyup="handler" />"#,
        sites: &["8 /* PROPS */, [\"onKeyup\"]"],
    },
    Case {
        name: "hydrating_key_event",
        src: r#"<div @keyup.enter="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]"],
    },
    Case {
        name: "hydrating_key_stop_event",
        src: r#"<div @keyup.enter.stop="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]"],
    },
    Case {
        name: "hydrating_key_event_plain",
        src: r#"<div @keyup="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]"],
    },
    Case {
        name: "need_patch",
        src: r#"<div ref="el"></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "native_v_model_props",
        src: r#"<input v-model="msg">"#,
        sites: &["8 /* PROPS */, [\"onUpdate:modelValue\"]"],
    },
    Case {
        name: "component_v_model_props",
        src: r#"<Foo v-model="msg" />"#,
        sites: &["8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]"],
    },
    Case {
        name: "named_component_v_model_props",
        src: r#"<Foo v-model:title="pageTitle" />"#,
        sites: &["8 /* PROPS */, [\"title\", \"onUpdate:title\"]"],
    },
    Case {
        name: "dynamic_component_v_model_full_props",
        src: r#"<Foo v-model:[field]="msg" />"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "dynamic_component_v_model_modifier_full_props",
        src: r#"<Foo v-model:[field].trim="msg" />"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "component_v_model_modifier_props",
        src: r#"<Foo v-model.lazy.trim="msg" />"#,
        sites: &["8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]"],
    },
    Case {
        name: "component_v_model_update_listener_order",
        src: r#"<Foo v-model="msg" @update:modelValue="track" />"#,
        sites: &["8 /* PROPS */, [\"modelValue\", \"onUpdate:modelValue\"]"],
    },
    Case {
        name: "component_listener_before_v_model_order",
        src: r#"<Foo @update:modelValue="track" v-model="msg" />"#,
        sites: &["8 /* PROPS */, [\"onUpdate:modelValue\", \"modelValue\"]"],
    },
    Case {
        name: "component_click_stop_props",
        src: r#"<Foo @click.stop="h" />"#,
        sites: &["8 /* PROPS */, [\"onClick\"]"],
    },
    Case {
        name: "directive_need_patch",
        src: r#"<div v-example></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "directive_text_need_patch",
        src: r#"<div v-example>{{ msg }}</div>"#,
        sites: &["1 /* TEXT */", "513 /* TEXT, NEED_PATCH */"],
    },
    Case {
        name: "dynamic_slots",
        src: r#"<Foo><template #header v-if="ok">x</template></Foo>"#,
        sites: &["2 /* DYNAMIC */", "1024 /* DYNAMIC_SLOTS */"],
    },
    Case {
        name: "dynamic_slots_builtin",
        src: "<KeepAlive><Foo /></KeepAlive>",
        sites: &["1024 /* DYNAMIC_SLOTS */"],
    },
    Case {
        name: "dynamic_slot_name",
        src: r#"<Foo><template #[name]>x</template></Foo>"#,
        sites: &["2 /* DYNAMIC */", "1024 /* DYNAMIC_SLOTS */"],
    },
    Case {
        name: "text_and_class",
        src: r#"<div :class="cls">{{ msg }}</div>"#,
        sites: &["3 /* TEXT, CLASS */"],
    },
    Case {
        name: "object_bind_named_prop",
        src: r#"<div :id="foo" v-bind="obj"></div>"#,
        sites: &["16 /* FULL_PROPS */, [\"id\"]"],
    },
    Case {
        name: "object_bind_keyup_hydration",
        src: r#"<div @keyup="handler" v-bind="obj"></div>"#,
        sites: &["48 /* FULL_PROPS, NEED_HYDRATION */, [\"onKeyup\"]"],
    },
    Case {
        name: "component_object_bind",
        src: r#"<Foo v-bind="obj" />"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "component_object_bind_static_literal_prop",
        src: r#"<Foo v-bind="obj" :props-loading="true" />"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "stable_fragment",
        src: "<div></div><span></span>",
        sites: &["64 /* STABLE_FRAGMENT */"],
    },
    Case {
        name: "keyed_fragment",
        src: r#"<div v-for="item in list" :key="item.id">{{ item.label }}</div>"#,
        sites: &["1 /* TEXT */", "128 /* KEYED_FRAGMENT */"],
    },
    Case {
        name: "unkeyed_fragment",
        src: r#"<div v-for="item in list">{{ item.label }}</div>"#,
        sites: &["1 /* TEXT */", "256 /* UNKEYED_FRAGMENT */"],
    },
    Case {
        name: "nested_dynamic_slot_fragment",
        src: r#"<div v-for="i in n"><Foo>hello</Foo></div>"#,
        sites: &[
            "2 /* DYNAMIC */",
            "1024 /* DYNAMIC_SLOTS */",
            "256 /* UNKEYED_FRAGMENT */",
        ],
    },
];

#[test]
fn s2_patch_flags_match_the_shipped_dom_lane_per_node() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);

    let mut mismatches = Vec::new();
    for case in CASES {
        let expected: Vec<_> = case.sites.iter().map(|site| site.to_string()).collect();
        let old = support::shipped(case.src);
        let allocator = Allocator::new();
        let new = emit_dom_source(&allocator, case.src)
            .unwrap_or_else(|error| panic!("{}: S2 emit refused: {error:?}", case.name))
            .assembled();
        let old_sites = support::patch_sites(&old);
        let new_sites = support::patch_sites(&new);

        if old_sites != expected || new_sites != expected {
            mismatches.push(format!(
                "{}: expected={expected:?} old={old_sites:?} new={new_sites:?}",
                case.name
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
