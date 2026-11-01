//! These combined-rule inputs are new controls, not historical oracle rows.
use super::support::{NUMERIC, ORDERS, RULE, case, pair};
use crate::{HelpLevel, Locale, Severity};

fn ordered(input: &super::support::Case<'_>, expected: &[&str]) {
    let observed = pair(input);
    let names = observed
        .initial
        .diagnostics
        .iter()
        .map(|d| d.rule_name)
        .collect::<Vec<_>>();
    assert_eq!(names, expected);
    let applied = observed.applications.first().unwrap();
    assert_eq!(
        applied.source.as_str(),
        input.source.replacen(".13", ".enter", 1)
    );
    let numeric_index = names.iter().position(|name| *name == NUMERIC).unwrap();
    assert_eq!(applied.diagnostic_index, numeric_index);
    let after_names = applied
        .result
        .diagnostics
        .iter()
        .map(|d| d.rule_name)
        .collect::<Vec<_>>();
    assert_eq!(
        after_names,
        expected
            .iter()
            .copied()
            .filter(|name| *name != NUMERIC)
            .collect::<Vec<_>>()
    );
    let valid = observed
        .initial
        .diagnostics
        .iter()
        .find(|d| d.rule_name == RULE);
    if let Some(diagnostic) = valid {
        assert!(diagnostic.fix.is_none());
    }
}

#[test]
fn same_attribute_keeps_all_six_registry_orders_and_actual_numeric_fix_requery() {
    for order in ORDERS {
        for source in ["<div @click.native.13 />", "<div @click.13.native />"] {
            ordered(
                &case("new-three-rule-order", source, order, 3, 1),
                order.names(),
            );
        }
        let expected = order
            .names()
            .iter()
            .copied()
            .filter(|name| *name != RULE)
            .collect::<Vec<_>>();
        ordered(
            &case(
                "new-verb-preserves-other-rules",
                "<input @keyup.native.13.stop />",
                order,
                2,
                1,
            ),
            &expected,
        );
        for source in [
            "<input @keyup.native.0.13 />",
            "界🌸\r\n<div @keyup.native.enter />",
        ] {
            let observed = pair(&case("new-first-zero-or-named", source, order, 2, 0));
            assert_eq!(
                observed
                    .initial
                    .diagnostics
                    .iter()
                    .map(|d| d.rule_name)
                    .collect::<Vec<_>>(),
                order
                    .names()
                    .iter()
                    .copied()
                    .filter(|name| *name != NUMERIC)
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn overlapping_rules_keep_complete_policy_packets_in_every_registry_order() {
    for order in ORDERS {
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
                for severity in [Severity::Error, Severity::Warning] {
                    let mut input = case(
                        "new-three-rule-policy",
                        "<div @click.native.13 />",
                        order,
                        3,
                        1,
                    );
                    input.locale = locale;
                    input.help = help;
                    input.severity = Some(severity);
                    ordered(&input, order.names());
                }
            }
        }
    }
}
