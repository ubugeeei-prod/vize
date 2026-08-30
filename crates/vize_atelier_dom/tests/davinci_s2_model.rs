//! P2-11 `v-model` witness: native `withDirectives` + `vModelText`-family
//! helpers, component `modelValue` / `onUpdate:` product props, compared
//! **byte-for-byte** including helper usage and hoists.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("input", r#"<input v-model="msg">"#),
    ("textarea", r#"<textarea v-model="msg"></textarea>"#),
    ("select", r#"<select v-model="msg"></select>"#),
    (
        "select_dyn",
        r#"<select v-model="msg"><option :value="v">{{ v }}</option></select>"#,
    ),
    (
        "select_static_then_for",
        r#"<select v-model="msg"><option value="">Select</option><option v-for="(item, index) of items" :key="index" :value="item.value">{{ item.label }}</option></select>"#,
    ),
    ("checkbox", r#"<input type="checkbox" v-model="ok">"#),
    ("radio", r#"<input type="radio" v-model="choice">"#),
    ("file", r#"<input type="file" v-model="msg">"#),
    ("lazy", r#"<input v-model.lazy="msg">"#),
    ("number", r#"<input v-model.number="msg">"#),
    ("trim", r#"<input v-model.trim="msg">"#),
    ("trim_number", r#"<input v-model.trim.number="msg">"#),
    ("nested", r#"<div><input v-model="msg"></div>"#),
    ("member", r#"<input v-model="form.name">"#),
    ("static_class", r#"<input class="foo" v-model="msg">"#),
    ("bind_id", r#"<input :id="id" v-model="msg">"#),
    ("id_then_model", r#"<input id="x" v-model="msg">"#),
    ("model_then_id", r#"<input v-model="msg" id="x">"#),
    ("class_bind", r#"<input :class="c" v-model="msg">"#),
    ("style_bind", r#"<input :style="s" v-model="msg">"#),
    ("spread_then", r#"<input v-bind="obj" v-model="msg">"#),
    ("then_spread", r#"<input v-model="msg" v-bind="obj">"#),
    ("vif", r#"<input v-if="ok" v-model="msg">"#),
    (
        "vfor",
        r#"<input v-for="item in list" v-model="item.value" :key="item.id">"#,
    ),
    ("div", r#"<div v-model="msg"></div>"#),
    ("fragment", r#"<input v-model="a"><input v-model="b">"#),
    ("comp", r#"<Foo v-model="msg" />"#),
    ("comp_arg", r#"<Foo v-model:title="pageTitle" />"#),
    (
        "comp_kebab_arg",
        r#"<Foo v-model:auto-send="autoSendEnabled" />"#,
    ),
    ("comp_dynamic_arg", r#"<Foo v-model:[field]="msg" />"#),
    (
        "comp_dynamic_arg_mod",
        r#"<Foo v-model:[field].trim="msg" />"#,
    ),
    ("comp_props", r#"<Foo v-model="source" :language="lang" />"#),
    (
        "comp_lang_first",
        r#"<Foo :language="lang" v-model="source" />"#,
    ),
    ("comp_mods", r#"<Foo v-model.lazy.trim="msg" />"#),
    ("comp_named_mods", r#"<Foo v-model:title.lazy="msg" />"#),
    ("multi_comp", r#"<Foo v-model="a" v-model:title="b" />"#),
    ("slot_comp", r#"<Foo v-model="msg">hello</Foo>"#),
    ("nested_comp", r#"<div><Foo v-model="msg" /></div>"#),
    ("vif_comp", r#"<Foo v-if="ok" v-model="msg" />"#),
    (
        "vfor_comp",
        r#"<Foo v-for="item in list" v-model="item.value" :key="item.id" />"#,
    ),
    ("spread_comp", r#"<Foo v-bind="obj" v-model="msg" />"#),
    ("click_then_model", r#"<input @click="h" v-model="msg">"#),
    ("model_then_click", r#"<input v-model="msg" @click="h">"#),
];

#[test]
fn s2_v_model_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
