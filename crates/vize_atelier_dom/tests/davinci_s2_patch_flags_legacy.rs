//! P2-11 legacy patch-flag witness for S2 DOM parity.
#![cfg(feature = "legacy")]
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_s0::Allocator;
use vize_s0::config::VueVersion;
use vize_s1_to_s2::{LegacyCaps, emit_dom_source_with_caps};

struct Case {
    name: &'static str,
    src: &'static str,
    sites: &'static [&'static str],
}

#[test]
fn s2_vue2_filter_patch_flags_match_the_shipped_dom_lane_per_node() {
    const FILTER_CASES: &[Case] = &[
        Case {
            name: "filter_text_child",
            src: "<div>{{ 1 | double }}</div>",
            sites: &["1 /* TEXT */"],
        },
        Case {
            name: "filter_bind_prop",
            src: r#"<div :id="1 | formatId"></div>"#,
            sites: &["8 /* PROPS */, [\"id\"]"],
        },
        Case {
            name: "filter_component_prop",
            src: r#"<Foo :value="1 | formatId" />"#,
            sites: &["8 /* PROPS */, [\"value\"]"],
        },
        Case {
            name: "filter_default_slot",
            src: "<Foo>{{ 1 | cap }}</Foo>",
            sites: &["1 /* TEXT */", "1 /* STABLE */"],
        },
    ];
    support::assert_s2_matches_prefixed_shipped_literals_with_dialect(
        &FILTER_CASES
            .iter()
            .map(|case| (case.name, case.src))
            .collect::<Vec<_>>(),
        VueVersion::V2,
    );

    let mut mismatches = Vec::new();
    for case in FILTER_CASES {
        let expected: Vec<_> = case.sites.iter().map(|site| site.to_string()).collect();
        let old = support::shipped_prefixed_with_dialect(case.src, VueVersion::V2);
        let allocator = Allocator::new();
        let new = emit_dom_source_with_caps(
            &allocator,
            case.src,
            LegacyCaps::for_version(VueVersion::V2),
        )
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
        "Vue 2 filter patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}
