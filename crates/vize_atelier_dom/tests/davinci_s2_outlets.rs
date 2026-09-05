//! P2-11 slot-outlet witness: `_renderSlot`, fallback, camelized props,
//! named and dynamic event props, `v-if` / `v-for` outlets, and
//! `_: 3 /* FORWARDED */`, compared
//! **byte-for-byte** including helper usage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s0::Allocator;
use vize_s0::config::VueVersion;
use vize_s1_to_s2::emit_dom_source;

struct Case {
    name: &'static str,
    src: &'static str,
    sites: &'static [&'static str],
}

#[test]
fn s2_slot_outlets_match_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped(support::battery::outlets::OUTLET_BATTERY);
}

const PATCH_SITE_CASES: &[Case] = &[
    Case {
        name: "dynamic_name",
        src: r#"<slot :name="n"></slot>"#,
        sites: &[],
    },
    Case {
        name: "bind_prop",
        src: r#"<slot :foo="bar"></slot>"#,
        sites: &[],
    },
    Case {
        name: "dynamic_event",
        src: r#"<slot @[event]="handler"></slot>"#,
        sites: &[],
    },
    Case {
        name: "dynamic_prop_and_dynamic_event",
        src: r#"<slot :[propKey]="value" @[event]="handler"></slot>"#,
        sites: &[],
    },
    Case {
        name: "object_bind",
        src: r#"<slot v-bind="obj"></slot>"#,
        sites: &[],
    },
    Case {
        name: "object_on_modifier",
        src: r#"<slot v-on.once="listeners"></slot>"#,
        sites: &[],
    },
    Case {
        name: "vif",
        src: r#"<slot v-if="ok"></slot>"#,
        sites: &[],
    },
    Case {
        name: "vif_else",
        src: r#"<slot v-if="a"></slot><slot v-else></slot>"#,
        sites: &[],
    },
    Case {
        name: "vfor",
        src: r#"<slot v-for="i in n"></slot>"#,
        sites: &["256 /* UNKEYED_FRAGMENT */"],
    },
    Case {
        name: "vfor_dynamic_event_local_name",
        src: r#"<slot v-for="item in items" @[item.event]="item.handler"></slot>"#,
        sites: &["256 /* UNKEYED_FRAGMENT */"],
    },
    Case {
        name: "forwarded",
        src: "<Foo><slot></slot></Foo>",
        sites: &["3 /* FORWARDED */"],
    },
    Case {
        name: "forwarded_nested",
        src: "<Foo><div><slot></slot></div></Foo>",
        sites: &["3 /* FORWARDED */"],
    },
    Case {
        name: "scoped_forwarded",
        src: r#"<Bar v-slot="p"><Foo><slot></slot></Foo></Bar>"#,
        sites: &[
            "2 /* DYNAMIC */",
            "1024 /* DYNAMIC_SLOTS */",
            "3 /* FORWARDED */",
        ],
    },
    Case {
        name: "scoped_forwarded_dynamic_event",
        src: r#"<Bar v-slot="{ row }"><Foo><slot @[row.event]="row.handler"></slot></Foo></Bar>"#,
        sites: &[
            "2 /* DYNAMIC */",
            "1024 /* DYNAMIC_SLOTS */",
            "3 /* FORWARDED */",
        ],
    },
    Case {
        name: "conditional_forwarded_dynamic_event",
        src: r#"<Foo><slot v-if="ok" @[event]="handler"></slot></Foo>"#,
        sites: &["3 /* FORWARDED */"],
    },
    Case {
        name: "named_mixed_props",
        src: r#"<slot name="header" foo="1" :bar="b"></slot>"#,
        sites: &[],
    },
];

#[test]
fn s2_slot_outlet_patch_sites_match_the_shipped_dom_lane_per_node() {
    let battery: Vec<_> = PATCH_SITE_CASES
        .iter()
        .map(|case| (case.name, case.src))
        .collect();
    support::assert_s2_matches_shipped(&battery);

    let mut mismatches = Vec::new();
    for case in PATCH_SITE_CASES {
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
        "slot outlet patch site mismatches:\n{}",
        mismatches.join("\n")
    );
}

const VUE2_BATTERY: &[(&str, &str)] = &[(
    "vue2_native_modifier",
    r#"<slot @click.native="handler"></slot>"#,
)];

#[test]
fn s2_vue2_slot_outlet_v_on_matches_the_shipped_dom_lane_byte_for_byte() {
    support::assert_s2_matches_shipped_with_dialect(VUE2_BATTERY, VueVersion::V2);
}
