//! Static literal bind patch-flag witnesses split from the broad S2 corpus.

mod support;

use vize_s0::Allocator;
use vize_s1_to_s2::emit_dom_source;

struct Case {
    name: &'static str,
    src: &'static str,
}

const CASES: &[Case] = &[
    Case {
        name: "static_literal_prop",
        src: "<div :id=\"'fixed'\"></div>",
    },
    Case {
        name: "component_static_literal_prop",
        src: "<Foo :side-offset=\"4\" />",
    },
    Case {
        name: "component_static_class_array",
        src: "<Foo :class=\"['card']\" />",
    },
];

#[test]
fn static_literal_bind_patch_flags_match_the_shipped_dom_lane() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);

    for case in CASES {
        let allocator = Allocator::new();
        let old = support::shipped(case.src);
        let new = emit_dom_source(&allocator, case.src).expect(case.name);

        assert!(
            support::patch_sites(&old).is_empty(),
            "{} shipped output should not mark static literal binds as patch sites",
            case.name
        );
        assert!(
            support::patch_sites(new.assembled().as_str()).is_empty(),
            "{} S2 output should not mark static literal binds as patch sites",
            case.name
        );
    }
}
