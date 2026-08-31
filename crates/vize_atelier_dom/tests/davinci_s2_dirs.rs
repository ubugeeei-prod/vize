//! P2-11 custom-directive witness: `resolveDirective` + `withDirectives`,
//! compared **byte-for-byte** including helper usage, asset order, and
//! NEED_PATCH.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    ("empty", r#"<div v-example></div>"#),
    ("text", r#"<div v-example>hello</div>"#),
    ("interp", r#"<div v-example>{{ msg }}</div>"#),
    ("compound", r#"<div v-example>hello {{ msg }}</div>"#),
    ("value", r#"<div v-example="val"></div>"#),
    ("arg", r#"<div v-example:arg="val"></div>"#),
    ("arg_only", r#"<div v-example:arg></div>"#),
    ("mod", r#"<div v-example.foo></div>"#),
    ("mods_value", r#"<div v-example.foo.bar="val"></div>"#),
    ("dyn_arg", r#"<div v-example:[dyn]="val"></div>"#),
    ("two", r#"<div v-pin v-foo></div>"#),
    (
        "dynamic_component_custom_directive_keeps_need_patch",
        r#"<component :is="view" v-example />"#,
    ),
    (
        "dynamic_component_custom_directive_static_props_keeps_need_patch",
        r#"<component :is="copied ? 'CheckOutlined' : 'SnippetsOutlined'" key="copy" class="code-action" v-clipboard:copy="sourceCode" v-clipboard:success="handleCodeCopied" />"#,
    ),
    ("kebab", r#"<div v-my-dir></div>"#),
    ("id", r#"<div id="x" v-example></div>"#),
    ("bind_id", r#"<div :id="id" v-example></div>"#),
    ("click", r#"<div @click="h" v-example></div>"#),
    ("spread", r#"<div v-bind="obj" v-example></div>"#),
    ("class_bind", r#"<div :class="c" v-example></div>"#),
    ("nested", r#"<div><span v-example></span></div>"#),
    ("nested_text", r#"<div><span v-example>hello</span></div>"#),
    (
        "static_child",
        r#"<div v-example><span id="">content</span></div>"#,
    ),
    ("nested_dir", r#"<div v-outer><span v-inner></span></div>"#),
    ("vif", r#"<div v-if="ok" v-example></div>"#),
    ("vif_text", r#"<div v-if="ok" v-example>hello</div>"#),
    ("vfor", r#"<div v-for="i in n" v-example></div>"#),
    ("vfor_text", r#"<div v-for="i in n" v-example>hello</div>"#),
    (
        "vfor_interp",
        r#"<div v-for="i in n" v-example>{{ msg }}</div>"#,
    ),
    (
        "vfor_key",
        r#"<div v-for="i in n" :key="i" v-example></div>"#,
    ),
    ("vfor_id", r#"<div v-for="i in n" :id="i" v-example></div>"#),
    ("model", r#"<input v-model="x" v-example>"#),
    ("example_then_model", r#"<input v-example v-model="x">"#),
    ("div_model", r#"<div v-model="x" v-example></div>"#),
    (
        "nested_model",
        r#"<div v-example><input v-model="x"></div>"#,
    ),
    ("comp", r#"<Foo v-example />"#),
    ("comp_text", r#"<Foo v-example>hello</Foo>"#),
    ("comp_model", r#"<Foo v-model="x" v-example />"#),
    ("comp_nested", r#"<div><Foo v-example /></div>"#),
    ("vif_comp", r#"<Foo v-if="ok" v-example />"#),
    ("vfor_comp", r#"<Foo v-for="i in n" v-example />"#),
    (
        "vfor_comp_key",
        r#"<Foo v-for="i in n" :key="i" v-example />"#,
    ),
    ("fragment", r#"<div v-example></div><p v-other></p>"#),
];

#[test]
fn s2_custom_dirs_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
