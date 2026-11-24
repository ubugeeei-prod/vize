//! The shared alias scan against the per-alias scans it replaced.

use super::{Buf, Helper, OrderCache};

#[test]
fn shared_scan_matches_each_alias_scan_across_lexical_boundaries() {
    for (hoist, code) in [
        ("", "_createVNode(a); _openBlock()"),
        ("_normalizeProps (p)", "x._createVNode(a), _createVNode (b)"),
        (
            "'_mergeProps(' /* _toDisplayString( */",
            "_mergeProps(x) // _openBlock(\n_openBlock()",
        ),
        (
            "`${_toDisplayString(a)}`",
            "a_createVNode(), $_createBlock(), _createBlockX(), _createBlock\n(x)",
        ),
        ("", "\"esc\\\"_renderList(\" _renderList(l)"),
    ] {
        let mut buf = Buf::new(false);
        for helper in Helper::ALL {
            buf.use_helper(helper);
        }
        if !hoist.is_empty() {
            buf.push_hoist(hoist.into());
        }
        buf.push(code);
        // The scan stops at each answer and resumes for the next
        // question, so the order of the questions must not matter.
        let mut forward = Helper::ALL.to_vec();
        let mut backward = forward.clone();
        backward.reverse();
        forward.rotate_left(7);
        for questions in [Helper::ALL.to_vec(), backward, forward] {
            let mut cache = OrderCache {
                rank_five_keys: [None; 8],
                alias_mask: super::ALL_ALIASES,
                alias_scan: None,
            };
            for helper in questions {
                assert_eq!(
                    buf.first_alias_position::<true>(helper, &mut cache),
                    buf.scan_alias_position(helper),
                    "{} in {hoist:?} + {code:?}",
                    helper.alias(),
                );
            }
        }
    }
}

#[test]
fn large_helper_module_keeps_ordinary_order_with_one_scan() {
    let mut buf = Buf::new(false);
    for helper in Helper::ALL.into_iter().rev() {
        buf.use_helper(helper);
        buf.push(helper.alias());
        buf.push("();");
    }
    buf.push("/* _mergeProps() */ '_normalizeProps()' object._camelize();");
    buf.push(&"x".repeat(65_536));

    let shared = buf.ordered_helpers();
    let mut listed = Helper::ALL.to_vec();
    listed.reverse();
    let ordinary = buf.sort_helpers::<false>(listed, 0);

    assert!(
        shared
            .iter()
            .map(|helper| helper.bit())
            .eq(ordinary.iter().map(|helper| helper.bit()))
    );
    let mut cache = OrderCache {
        rank_five_keys: [None; 8],
        alias_mask: buf.used,
        alias_scan: None,
    };
    assert!(
        buf.first_alias_position::<true>(Helper::MergeProps, &mut cache)
            .is_some()
    );
    assert_eq!(
        buf.first_alias_position::<true>(Helper::CreateVNode, &mut cache),
        buf.scan_alias_position(Helper::CreateVNode)
    );
}

/// The one shared scan is the only product path now, at every module
/// size. Generated modules mix used, preferred, called, hoisted, quoted
/// and never-called helpers; each must order exactly as the per-alias
/// scans order it.
#[test]
fn one_scan_orders_generated_modules_as_per_alias_scans_do() {
    // Fixed-seed xorshift: the corpus is the same on every run.
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = |bound: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state % bound as u64) as usize
    };
    let mut orders_with_alias_keys = 0_usize;
    for _ in 0..4_000 {
        let mut buf = Buf::new(false);
        let mut chosen = std::vec::Vec::new();
        for _ in 0..next(14) {
            let helper = Helper::ALL[next(Helper::ALL.len())];
            chosen.push(helper);
            match next(5) {
                0 => buf.prefer(helper),
                1 => {
                    buf.use_helper(helper);
                    let mut hoist = vize_l0::String::from(helper.alias());
                    hoist.push_str("(h)");
                    buf.push_hoist(hoist);
                }
                2 => {
                    buf.use_helper(helper);
                    buf.push("'");
                    buf.push(helper.alias());
                    buf.push("(' + ");
                }
                3 => buf.use_helper(helper),
                _ => {
                    buf.use_helper(helper);
                    buf.push(helper.alias());
                    buf.push(if next(2) == 0 { "(x), " } else { " (x)\n" });
                }
            }
        }
        let per_alias = buf.sort_helpers::<false>(buf.listed_helpers(), 0);
        let ordered = buf.ordered_helpers();
        assert!(
            ordered
                .iter()
                .map(|helper| helper.bit())
                .eq(per_alias.iter().map(|helper| helper.bit())),
            "{:?}",
            buf.code
        );
        orders_with_alias_keys += usize::from(
            chosen
                .iter()
                .filter(|helper| matches!(helper.rank(), 2 | 5))
                .count()
                > 1,
        );
    }
    // The corpus is not vacuous: many modules need alias positions.
    assert!(orders_with_alias_keys > 1_000, "{orders_with_alias_keys}");
}

#[test]
fn direct_alias_lookup_covers_every_helper() {
    for helper in Helper::ALL {
        assert_eq!(
            Helper::from_alias(helper.alias()).map(Helper::bit),
            Some(helper.bit())
        );
    }
    assert!(Helper::from_alias("_notAHelper").is_none());
}
