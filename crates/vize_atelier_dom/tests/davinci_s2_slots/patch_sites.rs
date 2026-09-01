use super::*;

const DYN_SLOTS: &[&str] = &["2 /* DYNAMIC */", "1024 /* DYNAMIC_SLOTS */"];
const TEXT_SLOTS: &[&str] = &[
    "1 /* TEXT */",
    "2 /* DYNAMIC */",
    "1024 /* DYNAMIC_SLOTS */",
];
const UNWRAPPED_IF_SITES: &[&str] = &["64 /* STABLE_FRAGMENT */", "1 /* STABLE */"];
const UNWRAPPED_FOR_SITES: &[&str] = &[
    "64 /* STABLE_FRAGMENT */",
    "256 /* UNKEYED_FRAGMENT */",
    "1 /* STABLE */",
];

const PATCH_SITE_CASES: &[(&str, &str, &[&str])] = &[
    (
        "create_slots_if",
        r#"<Foo><template #header v-if="ok">x</template></Foo>"#,
        DYN_SLOTS,
    ),
    (
        "create_slots_if_v_once",
        r#"<Foo><template #header v-if="ok" v-once>x</template></Foo>"#,
        DYN_SLOTS,
    ),
    (
        "create_slots_for",
        r#"<Foo><template v-for="i in n" #header>x</template></Foo>"#,
        DYN_SLOTS,
    ),
    (
        "create_slots_for_v_memo",
        r#"<Foo><template v-for="i in n" #header v-memo="[i]">x</template></Foo>"#,
        DYN_SLOTS,
    ),
    (
        "create_slots_dynamic_name",
        r#"<Foo><template #[name] v-if="ok">x</template></Foo>"#,
        DYN_SLOTS,
    ),
    (
        "create_slots_default_interp",
        r#"<Foo>hello {{ msg }}<template #header v-if="ok">x</template></Foo>"#,
        TEXT_SLOTS,
    ),
    (
        "unwrapped_if_nested_slot_keeps_siblings",
        r#"<Foo><template v-if="ok"><span>x</span><template #header>y</template></template></Foo>"#,
        UNWRAPPED_IF_SITES,
    ),
    (
        "unwrapped_for_nested_slot_keeps_siblings",
        r#"<Foo><template v-for="i in n"><span>x</span><template #header>y</template></template></Foo>"#,
        UNWRAPPED_FOR_SITES,
    ),
];

#[test]
fn s2_create_slots_patch_sites_match_the_shipped_dom_lane_per_node() {
    let battery: Vec<_> = PATCH_SITE_CASES
        .iter()
        .map(|(name, src, _)| (*name, *src))
        .collect();
    support::assert_s2_matches_shipped(&battery);

    let mut mismatches = Vec::new();
    for (name, src, sites) in PATCH_SITE_CASES {
        let expected: Vec<_> = sites.iter().map(|site| site.to_string()).collect();
        let old = support::shipped(src);
        let allocator = Allocator::new();
        let new = emit_dom_source(&allocator, src)
            .unwrap_or_else(|error| panic!("{name}: S2 emit refused: {error:?}"))
            .assembled();
        let old_sites = support::patch_sites(&old);
        let new_sites = support::patch_sites(&new);

        if old_sites != expected || new_sites != expected {
            mismatches.push(format!(
                "{name}: expected={expected:?} old={old_sites:?} new={new_sites:?}",
            ));
        }
    }
    let report = mismatches.join("\n");
    assert!(mismatches.is_empty(), "patch site mismatches:\n{report}");
}
