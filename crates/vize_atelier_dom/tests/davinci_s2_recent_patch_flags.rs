//! P2-11 recent-surface patch-flag witness: the S2 DOM lane keeps the
//! shipped per-node flags for the late directive/object-spread increments.

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
        name: "v_show_need_patch",
        src: r#"<div v-show="visible">content</div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "v_show_text_need_patch",
        src: r#"<div v-show="visible">{{ msg }}</div>"#,
        sites: &["513 /* TEXT, NEED_PATCH */"],
    },
    Case {
        name: "v_html_props",
        src: r#"<div v-html="raw"></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "v_html_style_props",
        src: r#"<div v-html="raw" :style="style"></div>"#,
        sites: &["12 /* STYLE, PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "v_text_props",
        src: r#"<div v-text="raw"></div>"#,
        sites: &["8 /* PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "v_text_style_props",
        src: r#"<div v-text="raw" :style="style"></div>"#,
        sites: &["12 /* STYLE, PROPS */, [\"textContent\"]"],
    },
    Case {
        name: "v_cloak_noop",
        src: r#"<div v-cloak></div>"#,
        sites: &[],
    },
    Case {
        name: "v_cloak_dynamic_prop",
        src: r#"<div v-cloak :id="id"></div>"#,
        sites: &["8 /* PROPS */, [\"id\"]"],
    },
    Case {
        name: "object_bind_prop_modifier_full_props",
        src: r#"<div v-bind.prop="bag"></div>"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "object_bind_camel_modifier_full_props",
        src: r#"<div v-bind.camel="bag"></div>"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "object_on_once_full_props",
        src: r#"<div v-on.once="handlers"></div>"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "component_object_on_modifier_full_props",
        src: r#"<Foo v-on.once.capture="handlers" />"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "component_object_bind_dynamic_style",
        src: r#"<Foo v-bind="obj" :style="style" />"#,
        sites: &["16 /* FULL_PROPS */, [\"style\"]"],
    },
    Case {
        name: "component_object_bind_dynamic_class",
        src: r#"<Foo v-bind="obj" :class="klass" />"#,
        sites: &["16 /* FULL_PROPS */, [\"class\"]"],
    },
];

#[test]
fn s2_recent_directive_patch_flags_match_the_shipped_dom_lane_per_node() {
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
        "recent directive patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
