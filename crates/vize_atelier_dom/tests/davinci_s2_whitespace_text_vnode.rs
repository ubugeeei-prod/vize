//! P2-11 DOM corpus parity pins for whitespace-only text vnode spelling.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

const BATTERY: &[(&str, &str)] = &[
    (
        "inline_element_space_uses_empty_create_text_call",
        r#"<p><sup>text</sup> <code>^text^</code></p>"#,
    ),
    (
        "inline_element_space_before_br_uses_empty_create_text_call",
        r#"<p><em>does this help the user think better?</em> <br><br></p>"#,
    ),
    (
        "keyboard_shortcut_space_uses_empty_create_text_call",
        r#"<p><kbd>Ctrl</kbd> <kbd>K</kbd></p>"#,
    ),
    (
        "hoisted_static_child_space_keeps_shipped_explicit_create_text_call",
        r#"<div :id="id"><span><sup>text</sup> <code>^text^</code></span></div>"#,
    ),
    (
        "cached_static_child_space_uses_empty_create_text_call",
        r#"<div class="root">{{ msg }}<span><kbd>Ctrl</kbd> <kbd>K</kbd></span></div>"#,
    ),
    (
        "conditional_component_space_before_icon_matches_shipped",
        r#"<div><Icon v-if="item.enabled" :name="item.enabledIcon" /> <Icon :name="item.icon" />{{ item.label }}</div>"#,
    ),
    (
        "implicit_slot_element_if_space_before_component_matches_shipped",
        r#"<NuxtLink><span v-if="mark.hasMark(item.newId)" class="bubble"></span> <Icon :name="item.icon" />{{ item.label }}</NuxtLink>"#,
    ),
    (
        "implicit_component_nbsp_default_slot_drops_like_shipped",
        r#"<Text>&nbsp;</Text>"#,
    ),
    (
        "implicit_component_numeric_nbsp_default_slot_drops_like_shipped",
        r#"<Text>&#160;</Text>"#,
    ),
    (
        "named_slot_element_if_space_before_component_matches_shipped",
        r#"<Foo><template #default><span v-if="ok" class="bubble"></span> <Icon :name="item.icon" />{{ item.label }}</template></Foo>"#,
    ),
    (
        "commented_element_gap_newline_drops_pre_comment_space",
        "<ul>\n  <li>{{ a }} {{ b }}</li> <!--c-->\n  <li>{{ c }} {{ d }}</li>\n</ul>",
    ),
    (
        "nested_once_compound_keeps_trailing_space_in_one_text_vnode",
        r#"<div v-once><p>{{ a }} <span v-html="b"></span></p></div>"#,
    ),
    (
        "noscript_mixed_children_condense_like_the_shipped_lane",
        r#"<div><noscript class="banner">
  {{ a }}
  <br />
  <a href="x">{{ b }}</a>
</noscript></div>"#,
    ),
];

#[test]
fn s2_whitespace_text_vnodes_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(BATTERY);
}
