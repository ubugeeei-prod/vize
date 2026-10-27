//! P2-11 `v-once` witness: native element cache wrappers from L2,
//! compared **byte-for-byte** against the shipped DOM lane.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

use crate::support;

use vize_l0::Allocator;
use vize_l1_to_l2::emit_dom_source;

struct Case {
    name: &'static str,
    src: &'static str,
    sites: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "simple",
        src: r#"<div v-once>x</div>"#,
        sites: &[],
    },
    Case {
        name: "interpolation",
        src: r#"<div v-once>{{ msg }}</div>"#,
        sites: &["1 /* TEXT */"],
    },
    Case {
        name: "static_attr_interpolation",
        src: r#"<div v-once class="dialog-header title">{{ $t('earnedAchievement') }}</div>"#,
        sites: &["1 /* TEXT */"],
    },
    Case {
        name: "compound_static_tail_merges_inside_cache_initializer",
        src: r##"<strong v-once>{{ $t("createdBy") }}:</strong>"##,
        sites: &["1 /* TEXT */"],
    },
    Case {
        name: "dynamic_class",
        src: r#"<div v-once :class="cls">content</div>"#,
        sites: &["2 /* CLASS */"],
    },
    Case {
        name: "dynamic_style",
        src: r#"<div v-once :style="style">content</div>"#,
        sites: &["4 /* STYLE */"],
    },
    Case {
        name: "nested_static",
        src: r#"<div v-once><span>x</span></div>"#,
        sites: &[],
    },
    Case {
        name: "nested_dynamic",
        src: r#"<div v-once><span :class="cls">{{ msg }}</span></div>"#,
        sites: &["3 /* TEXT, CLASS */"],
    },
    Case {
        name: "html_binding_stays_in_cache_initializer",
        src: r##"<div v-once class="opt-out-description" v-html="$t('optOutOfClassesText', optOutLinks)"></div>"##,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "event_and_html_bindings_stay_in_cache_initializer",
        src: r###"<span v-once class="close-icon svg-icon inline icon-10" @click="close()" v-html="icons.close"></span>"###,
        sites: &["8 /* PROPS */, [\"onClick\", \"innerHTML\"]"],
    },
    Case {
        name: "static_dynamic_class_and_html_keep_props_flag",
        src: r#"<span v-once class="svg-icon inline icon-16 mr-1" :class="{ 'w-0': currencyClass === 'unlock'}" v-html="icons[currencyClass]"></span>"#,
        sites: &["10 /* CLASS, PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "html_only_cache_initializer_keeps_inner_html",
        src: r#"<div v-once v-html="raw"></div>"#,
        sites: &["8 /* PROPS */, [\"innerHTML\"]"],
    },
    Case {
        name: "custom_directive_is_wrapped_inside_cache",
        src: r#"<h3 v-once v-example="heading">{{ question }}</h3>"#,
        sites: &["1 /* TEXT */", "512 /* NEED_PATCH */"],
    },
    Case {
        name: "component_custom_directive_is_wrapped_inside_cache",
        src: r#"<Foo v-once v-example="heading" />"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "event_binding_stays_in_cache_initializer",
        src: r###"<button v-once class="btn" @click="closeWithAction()">{{ $t("viewAchievements") }}</button>"###,
        sites: &["1 /* TEXT */", "8 /* PROPS */, [\"onClick\"]"],
    },
    Case {
        name: "nested_html_inside_once_parent_still_emits_html_prop",
        src: r###"<div v-once><span v-html="icons.sparkles"></span><span class="text">{{ $t("yourRewards") }}</span></div>"###,
        sites: &["8 /* PROPS */, [\"innerHTML\"]", "1 /* TEXT */"],
    },
    Case {
        name: "v_if_branch",
        src: r#"<div v-if="ok" v-once>x</div>"#,
        sites: &[],
    },
    Case {
        name: "component",
        src: r#"<Foo v-once />"#,
        sites: &[],
    },
    Case {
        name: "component_with_props_and_slot",
        src: r#"<Foo v-once :title="title"><span>x</span></Foo>"#,
        sites: &["1 /* STABLE */", "8 /* PROPS */, [\"title\"]"],
    },
    Case {
        name: "inside_v_for",
        src: r#"<div v-for="item in items" :key="item.id"><span v-once>{{ item.static }}</span></div>"#,
        sites: &["1 /* TEXT */", "128 /* KEYED_FRAGMENT */"],
    },
    Case {
        name: "empty_for_item_once_props",
        src: r#"<pre v-for="_ in []" v-once></pre>"#,
        sites: &["256 /* UNKEYED_FRAGMENT */"],
    },
];

#[test]
fn l2_v_once_matches_the_shipped_dom_lane_byte_for_byte() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_l2_matches_shipped(&battery);
}

#[test]
fn l2_v_once_patch_flags_match_the_shipped_dom_lane_per_node() {
    let mut mismatches = Vec::new();
    for case in CASES {
        let expected: Vec<_> = case.sites.iter().map(|site| site.to_string()).collect();
        let old = support::shipped(case.src);
        let allocator = Allocator::new();
        let new = emit_dom_source(&allocator, case.src)
            .unwrap_or_else(|error| panic!("{}: L2 emit refused: {error:?}", case.name))
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
        "v-once patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
