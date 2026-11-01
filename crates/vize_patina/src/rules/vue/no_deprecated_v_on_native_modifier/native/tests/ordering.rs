use super::support::{Case, NUMERIC, Order, RULE, case, configured, observe, pair};
use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::{HelpLevel, Locale, Severity};

fn ordered(input: &Case<'_>) {
    let observed = pair(input);
    let names: Vec<_> = observed
        .initial
        .diagnostics
        .iter()
        .map(|d| d.rule_name.as_str())
        .collect();
    let (expected, index) = match input.registration {
        Order::NativeNumeric => ([RULE, NUMERIC], 1),
        Order::NumericNative => ([NUMERIC, RULE], 0),
        Order::NativeOnly => unreachable!(),
    };
    assert_eq!(names, expected);
    let native = observed
        .initial
        .diagnostics
        .iter()
        .find(|d| d.rule_name == RULE)
        .unwrap();
    assert!(native.fix.is_none());
    let applied = observed.applications.first().unwrap();
    assert_eq!(applied.diagnostic_index, index);
    let linter = configured(input.registration, input.locale, input.help, input.severity);
    assert_eq!(
        format!("{:#?}", applied.result),
        format!(
            "{:#?}",
            linter.lint_template(&applied.source, input.filename)
        )
    );
}

#[test]
fn same_attribute_registry_order_and_numeric_fix_requery_remain_whole() {
    for order in [Order::NativeNumeric, Order::NumericNative] {
        for source in [
            "<div @click.native.13=\"h\" />",
            "<div @click.13.native=\"h\" />",
            "<div @click.native.13 />",
            "<div @click.13.native=\"\" />",
            "界🌸\r\n<div @keyup.native.13.27=\"h\" />",
        ] {
            ordered(&case("same-attribute-order", source, order, 2, 1));
        }
        for source in [
            "<div @keyup.native.0.13=\"h\" />",
            "<div @keyup.native.enter=\"h\" />",
        ] {
            let observed = pair(&case("numeric-first-zero-or-named", source, order, 1, 0));
            assert_eq!(
                observed
                    .initial
                    .diagnostics
                    .first()
                    .unwrap()
                    .rule_name
                    .as_str(),
                RULE
            );
        }
    }
}

#[test]
fn same_attribute_policy_matrix_preserves_both_registry_orders() {
    for order in [Order::NativeNumeric, Order::NumericNative] {
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            for help in [HelpLevel::None, HelpLevel::Short, HelpLevel::Full] {
                for severity in [Severity::Error, Severity::Warning] {
                    let mut input = case(
                        "same-attribute-policy",
                        "<div @click.native.13=\"h\" />",
                        order,
                        2,
                        1,
                    );
                    input.locale = locale;
                    input.help = help;
                    input.severity = Some(severity);
                    ordered(&input);
                }
            }
        }
    }
}

#[test]
fn separate_event_attributes_refuse_the_whole_result_in_both_registry_orders() {
    for order in [
        Order::NativeOnly,
        Order::NativeNumeric,
        Order::NumericNative,
    ] {
        let linter = configured(order, Locale::En, HelpLevel::Full, None);
        for (source, second, diagnostics, fixes, native_count) in [
            (
                "<div @keyup.13=\"h\" @click.native=\"h\" />",
                "@click.native=\"h\"",
                2,
                1,
                1,
            ),
            (
                "<div @click.native=\"h\" @keyup.13=\"h\" />",
                "@keyup.13=\"h\"",
                2,
                1,
                1,
            ),
            (
                "<div @click.native.13=\"h\" @keyup.native.27=\"h\" />",
                "@keyup.native.27=\"h\"",
                4,
                2,
                2,
            ),
            (
                "界🌸\r\n<div @click.native.13=\"h\" @keyup.27.native=\"h\" />",
                "@keyup.27.native=\"h\"",
                4,
                2,
                2,
            ),
            (
                "<div @click.native=\"h\"/><div @keyup.13=\"h\" @click.native=\"h\"/>",
                "@click.native=\"h\"",
                3,
                1,
                2,
            ),
        ] {
            let (diagnostics, fixes) = if matches!(order, Order::NativeOnly) {
                (native_count, 0)
            } else {
                (diagnostics, fixes)
            };
            let input = case(
                "separate-attributes-explicitly-unsupported",
                source,
                order,
                diagnostics,
                fixes,
            );
            let legacy = observe(&linter, &input, false);
            let actual = linter.lint_native_template(source, input.filename);
            println!("refused_case:{input:#?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
            assert_eq!(legacy.initial.diagnostics.len(), diagnostics);
            assert_eq!(legacy.applications.len(), fixes);
            // The final occurrence belongs to the late element. Earlier root
            // or numeric-first callback findings remain unpublished on Err.
            let start = source.rfind(second).unwrap() as u32;
            let expected = Refusal::UnsupportedAttribute {
                span: vize_l0::Span::new(start, start + second.len() as u32),
            };
            assert_eq!(actual.unwrap_err(), expected);
        }
    }
}
