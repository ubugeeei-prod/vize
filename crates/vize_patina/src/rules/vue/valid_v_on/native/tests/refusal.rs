use super::support::{ORDERS, ORIGINAL, Order, RULE, case, configured, observe, pair_with, span};
use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::rules::vue::{NoDeprecatedHtmlElementIs, ValidVOn};
use crate::{Linter, RuleRegistry};
use vize_l0::config::VueVersion;

#[test]
fn original_clean_form_retains_the_existing_whole_owner_refusal() {
    let source = "<form @submit.prevent></form>";
    let input = case("old-original-clean-form", source, Order::ValidOnly, 0, 0);
    let linter = configured(&input);
    let legacy = observe(&linter, &input, false);
    let actual = linter.lint_native_template(source, input.filename);
    println!("original-form-refusal:{input:#?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
    assert!(legacy.initial.diagnostics.is_empty());
    assert!(legacy.applications.is_empty());
    assert_eq!(
        actual.unwrap_err(),
        Refusal::UnsupportedContext {
            span: span(source, "<form @submit.prevent>")
        }
    );
}

#[test]
fn second_on_or_any_value_refuses_wholesale_even_after_numeric_first_findings() {
    for order in ORDERS {
        let input = case("new-explicit-refusal", ORIGINAL, order, 0, 0);
        let linter = configured(&input);
        for (source, rejected) in [
            ("<div @keyup.13 @click.native />", "@click.native"),
            ("<div @click.native @keyup.13 />", "@keyup.13"),
            (
                "<div @click.native/><div @keyup.13 @click.native/>",
                "@click.native",
            ),
            ("<div @click.native.13=\"h\" />", "@click.native.13=\"h\""),
            ("<div @click=\"\" />", "@click=\"\""),
            ("<div @click.once=\"  \" />", "@click.once=\"  \""),
            ("<div @click=\"for\" />", "@click=\"for\""),
            ("<div @click=\"&#x66;or\" />", "@click=\"&#x66;or\""),
        ] {
            let mut input = case("new-whole-refusal", source, order, 0, 0);
            input.diagnostics = None;
            input.fixes = None;
            let legacy = observe(&linter, &input, false);
            let actual = linter.lint_native_template(source, input.filename);
            println!("refused_case:{input:#?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
            let start = source.rfind(rejected).unwrap() as u32;
            assert_eq!(
                actual.unwrap_err(),
                Refusal::UnsupportedAttribute {
                    span: vize_l0::Span::new(start, start + rejected.len() as u32)
                }
            );
        }
    }
}

#[test]
fn dynamic_missing_object_component_sfc_and_late_unknown_contexts_stay_unsupported() {
    use crate::native::NativeLintRefusal;
    let input = case("unsupported-contexts", ORIGINAL, Order::ValidOnly, 1, 0);
    let linter = configured(&input);
    for (source, head) in [
        ("<div @[event].native />", "@[event].native"),
        ("<div @.native />", "@.native"),
        ("<div v-on />", "v-on"),
        ("<div v-on=\"{ click: handler }\" />", "v-on"),
        ("<div @click.native./>", "@click.native."),
        ("<div @click.native/><div v-unknown=\"x\"/>", "v-unknown"),
    ] {
        let mut row = case("whole-header-refusal", source, Order::ValidOnly, 0, 0);
        row.diagnostics = None;
        row.fixes = None;
        let legacy = observe(&linter, &row, false);
        let actual = linter.lint_native_template(source, row.filename);
        println!("refused_case:{row:#?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
        assert_eq!(
            actual.unwrap_err(),
            Refusal::Header(NativeLintRefusal::UnsupportedDirective {
                span: span(source, head)
            })
        );
    }
    for (source, opening) in [
        (
            "<MyComponent @click.native/>",
            "<MyComponent @click.native/>",
        ),
        ("<template><div @click.native/></template>", "<template>"),
        ("<div @click.native/><MyComponent/>", "<MyComponent/>"),
    ] {
        let actual = linter.lint_native_template(source, input.filename);
        println!("new-owner-refusal source:{source:?}\nnative:{actual:#?}");
        assert_eq!(
            actual.unwrap_err(),
            Refusal::UnsupportedContext {
                span: span(source, opening)
            }
        );
    }
    let source = "<template><div @click.native /></template>";
    let legacy = linter.lint_sfc(source, input.filename);
    let actual = linter.lint_native_sfc(source, input.filename);
    println!("new-sfc-refusal source:{source:?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
    assert_eq!(
        actual.unwrap_err(),
        crate::native::sfc::NativeSfcLintRefusal::UnprovidedRule { rule: RULE.into() }
    );
}

#[test]
fn actual_configured_disabled_and_unprovided_instances_never_enable_a_rule() {
    let mut input = case(
        "disabled-with-other-real-event-rule",
        ORIGINAL,
        Order::ValidNativeNumeric,
        1,
        0,
    );
    input.disabled_rules = vec![RULE.into()];
    let linter = configured(&input);
    pair_with(&linter, &input);
    input.registration = Order::NativeOnly;
    input.disabled_rules.clear();
    input.custom_modifiers = Some(vec!["custom".into()]);
    let linter = configured(&input);
    assert!(!linter.registry().has_rule(RULE));
    pair_with(&linter, &input);
    let unknown = configured(&input).with_enabled_rules(Some(vec!["external/unknown".into()]));
    let actual = unknown.lint_native_template(ORIGINAL, input.filename);
    println!("unknown original:{input:#?}\nnative:{actual:#?}");
    assert_eq!(
        actual.unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "external/unknown".into()
        }
    );
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ValidVOn));
    registry.register(Box::new(NoDeprecatedHtmlElementIs));
    let actual = Linter::with_registry(registry).lint_native_template(ORIGINAL, input.filename);
    println!("mixed profile:{actual:#?}");
    assert_eq!(
        actual.unwrap_err(),
        Refusal::UnsupportedAttribute {
            span: span(ORIGINAL, "@click.native")
        }
    );
    let version = configured(&input).with_vue_version(Some(VueVersion::V2));
    let actual = version.lint_native_template(ORIGINAL, input.filename);
    println!("requested Vue2 original:{input:#?}\nnative:{actual:#?}");
    assert_eq!(
        actual.unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
    let vapor = configured(&input).with_vapor_mode(Some(true));
    let actual = vapor.lint_native_template(ORIGINAL, input.filename);
    println!("requested Vapor original:{input:#?}\nnative:{actual:#?}");
    assert_eq!(
        actual.unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true }
    );
}
