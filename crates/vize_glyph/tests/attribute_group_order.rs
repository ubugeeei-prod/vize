//! Authored #6098 boundary controls, not a reporter-supplied counterexample.

use vize_glyph::{AttributeSortOrder, FormatOptions, format_template};
use vize_l0::String;

const SOURCE: &str = r#"<div a-late="l" z-early="e"></div>"#;
const ORDERED: &str = r#"<div z-early="e" a-late="l"></div>"#;

fn groups(count: usize, matched: bool) -> Vec<Vec<String>> {
    let mut groups = vec![Vec::new(); count];
    if let Some(first) = groups.first_mut() {
        first.push("z-early".into());
    }
    if matched && count > 1 {
        groups.last_mut().unwrap().push("a-late".into());
    }
    groups
}

#[test]
fn custom_group_ordinals_and_unmatched_fallback_are_lossless() {
    let mut observations = Vec::new();
    for count in [0, 1, 255, 256, 257, 512] {
        for matched in [false, true] {
            for order in [
                AttributeSortOrder::Alphabetical,
                AttributeSortOrder::AsWritten,
            ] {
                let options = FormatOptions {
                    print_width: 200,
                    attribute_groups: Some(groups(count, matched)),
                    attribute_sort_order: order,
                    ..FormatOptions::default()
                };
                let first = format_template(SOURCE, &options);
                let second = first
                    .as_ref()
                    .ok()
                    .map(|value| format_template(value, &options));
                observations.push((count, matched, options, first, second));
            }
        }
    }
    // Retain every complete option/result before the first rank assertion.
    eprintln!("attribute-group-ordinal-observations:{observations:#?}");
    for (count, matched, _, first, second) in observations {
        let first = first.unwrap();
        assert_eq!(
            first.as_str(),
            if count == 0 { SOURCE } else { ORDERED },
            "group count={count}, last matched={matched}"
        );
        assert_eq!(second.unwrap().unwrap(), first);
    }
}

#[test]
fn custom_groups_keep_barriers_duplicates_first_match_and_disabled_sorting() {
    let wide = groups(257, true);
    let spread = r#"<div a-late="l" z-early="e" v-bind="attrs" a-late="r" z-early="s"></div>"#;
    let spread_expected =
        r#"<div z-early="e" a-late="l" v-bind="attrs" z-early="s" a-late="r"></div>"#;
    let event = r#"<div a-late="l" @click="run" z-early="e"></div>"#;
    let duplicate = r#"<div a-late="l" z-early="e" a-late="r"></div>"#;
    let cases = [
        ("spread", spread, spread_expected, Some(wide.clone()), true),
        ("event", event, event, Some(wide.clone()), true),
        ("duplicate", duplicate, duplicate, Some(wide.clone()), true),
        ("disabled", SOURCE, SOURCE, Some(wide), false),
        ("builtin-default", SOURCE, SOURCE, None, true),
        ("empty", SOURCE, SOURCE, Some(Vec::new()), true),
        (
            "first-match",
            SOURCE,
            SOURCE,
            Some(vec![vec!["*".into()], vec!["z-early".into()]]),
            true,
        ),
        (
            "prefix",
            SOURCE,
            ORDERED,
            Some(vec![vec!["z-*".into()], vec!["a-*".into()]]),
            true,
        ),
    ];
    let observations: Vec<_> = cases
        .into_iter()
        .map(
            |(name, source, expected, attribute_groups, sort_attributes)| {
                let options = FormatOptions {
                    print_width: 200,
                    attribute_groups,
                    sort_attributes,
                    ..FormatOptions::default()
                };
                let first = format_template(source, &options);
                let second = first
                    .as_ref()
                    .ok()
                    .map(|value| format_template(value, &options));
                (name, source, expected, options, first, second)
            },
        )
        .collect();
    eprintln!("attribute-group-boundary-observations:{observations:#?}");
    for (name, _, expected, _, first, second) in observations {
        let first = first.unwrap();
        assert_eq!(first.as_str(), expected, "{name}");
        assert_eq!(second.unwrap().unwrap(), first, "{name}: fixed point");
    }
}
