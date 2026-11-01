use super::support::{ORIGINAL, Order, RULE, case, configured, observe, span};
use crate::native::template::NativeTemplateLintRefusal as Refusal;
use crate::rules::vue::{NoDeprecatedHtmlElementIs, NoDeprecatedVOnNativeModifier};
use crate::{HelpLevel, Linter, Locale, RuleRegistry};
use vize_l0::config::VueVersion;

#[test]
fn original_components_and_sfc_remain_whole_refusals_without_rewritten_inputs() {
    let linter = configured(Order::NativeOnly, Locale::En, HelpLevel::Full, None);
    for source in [
        "<MyComponent @click.native=\"handler\" />",
        "<MyComponent v-on:click.native=\"handler\" />",
        "<MyComponent @click.native.stop=\"handler\" />",
    ] {
        let input = case(
            "unchanged-own-component-original",
            source,
            Order::NativeOnly,
            1,
            0,
        );
        let legacy = observe(&linter, &input, false);
        let actual = linter.lint_native_template(source, input.filename);
        println!("refused_case:{input:#?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
        assert_eq!(legacy.initial.diagnostics.len(), 1);
        assert_eq!(
            actual.unwrap_err(),
            Refusal::UnsupportedContext {
                span: span(source, source)
            }
        );
    }
    let source = "<template><MyComponent @click.native=\"handler\" /></template>";
    let legacy = linter.lint_sfc(source, "Original.vue");
    let actual = linter.lint_native_sfc(source, "Original.vue");
    println!("source:{source:?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
    assert_eq!(
        actual.unwrap_err(),
        crate::native::sfc::NativeSfcLintRefusal::UnprovidedRule { rule: RULE.into() }
    );
}

#[test]
fn late_unknown_dynamic_missing_and_foreign_owner_contexts_never_publish_pending_findings() {
    use crate::native::NativeLintRefusal;
    let linter = configured(Order::NativeOnly, Locale::En, HelpLevel::Full, None);
    for (source, head) in [
        ("<div @[event].native=\"h\" />", "@[event].native"),
        ("<div v-on.native=\"h\" />", "v-on.native"),
        ("<div @.native=\"h\" />", "@.native"),
        ("<div @click.native.=\"h\" />", "@click.native."),
        (
            "<div @click.native=\"h\"/><div v-unknown=\"x\"/>",
            "v-unknown",
        ),
    ] {
        let mut input = case("unsupported-header", source, Order::NativeOnly, 1, 0);
        input.diagnostics = None;
        input.fixes = None;
        let legacy = observe(&linter, &input, false);
        let actual = linter.lint_native_template(source, input.filename);
        println!("refused_case:{input:#?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
        assert_eq!(
            actual.unwrap_err(),
            Refusal::Header(NativeLintRefusal::UnsupportedDirective {
                span: span(source, head)
            })
        );
    }
    for (source, opening) in [
        (
            "<template><div @click.native=\"h\"/></template>",
            "<template>",
        ),
        ("<div @click.native=\"h\"/><MyComponent/>", "<MyComponent/>"),
    ] {
        let mut input = case("late-owner-refusal", source, Order::NativeOnly, 1, 0);
        input.diagnostics = None;
        input.fixes = None;
        let legacy = observe(&linter, &input, false);
        let actual = linter.lint_native_template(source, input.filename);
        println!("refused_case:{input:#?}\nlegacy:{legacy:#?}\nnative:{actual:#?}");
        assert_eq!(
            actual.unwrap_err(),
            Refusal::UnsupportedContext {
                span: span(source, opening)
            }
        );
    }
}

#[test]
fn actual_instance_disabled_unknown_mixed_profile_and_vue_version_authority_stay_closed() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoDeprecatedVOnNativeModifier));
    registry.register(Box::new(NoDeprecatedHtmlElementIs));
    assert_eq!(
        Linter::with_registry(registry)
            .lint_native_template(ORIGINAL, "Mixed.vue")
            .unwrap_err(),
        Refusal::UnsupportedAttribute {
            span: span(ORIGINAL, "@click.native")
        }
    );
    let unknown = configured(Order::NativeOnly, Locale::En, HelpLevel::Full, None)
        .with_enabled_rules(Some(vec!["external/unknown".into()]));
    assert_eq!(
        unknown
            .lint_native_template(ORIGINAL, "Unknown.vue")
            .unwrap_err(),
        Refusal::UnprovidedRule {
            rule: "external/unknown".into()
        }
    );
    let off = configured(Order::NativeOnly, Locale::En, HelpLevel::Full, None)
        .with_disabled_rules(vec![RULE.into()]);
    let source = "<div/>";
    assert_eq!(
        format!(
            "{:#?}",
            off.lint_native_template(source, "Off.vue").unwrap()
        ),
        format!("{:#?}", off.lint_template(source, "Off.vue"))
    );
    assert_eq!(
        off.lint_native_template(ORIGINAL, "Off.vue").unwrap_err(),
        Refusal::UnsupportedAttribute {
            span: span(ORIGINAL, "@click.native")
        }
    );
    assert_eq!(
        configured(Order::NativeOnly, Locale::En, HelpLevel::Full, None)
            .with_vue_version(Some(VueVersion::V2))
            .lint_native_template(ORIGINAL, "Vue2.vue")
            .unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
    assert_eq!(
        configured(Order::NativeOnly, Locale::En, HelpLevel::Full, None)
            .with_vapor_mode(Some(true))
            .lint_native_template(ORIGINAL, "Vapor.vue")
            .unwrap_err(),
        Refusal::UnsupportedVaporMode { requested: true }
    );
}
