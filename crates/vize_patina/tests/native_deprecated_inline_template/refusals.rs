use super::support::*;
use vize_l0::{Span, config::VueVersion};
use vize_patina::{
    HelpLevel, LintContext, Linter, Locale, Rule, RuleMeta, RuleRegistry, Severity,
    native::{
        NativeLintRefusal, sfc::NativeSfcLintRefusal,
        template::NativeTemplateLintRefusal as Refusal,
    },
    rules::vue::NoDeprecatedInlineTemplate,
};
use vize_relief::RootNode;

#[test]
fn original_component_snapshot_and_late_unsupported_children_keep_whole_refusals() {
    let configured = configured(Locale::En, HelpLevel::Full, Severity::Error);
    let original_component = "<MyComponent inline-template><div>Content</div></MyComponent>";
    let mut observations = vec![(
        observe(
            &configured,
            original_component,
            expected(vec![finding(
                Locale::En,
                HelpLevel::Full,
                Severity::Error,
                Span::new(13, 28),
            )]),
        ),
        Refusal::UnsupportedContext {
            span: Span::new(0, 29),
        },
    )];
    for (suffix, refusal) in [
        (
            "<Widget />",
            Refusal::UnsupportedContext {
                span: Span::new(34, 44),
            },
        ),
        (
            "<!-- later -->",
            Refusal::Comment {
                span: Span::new(34, 48),
            },
        ),
        (
            "{{ value }}",
            Refusal::Interpolation {
                span: Span::new(34, 45),
            },
        ),
        (
            "<span :title='value'/>",
            Refusal::UnsupportedAttribute {
                span: Span::new(40, 54),
            },
        ),
    ] {
        let source = format!("{ORIGINAL}{suffix}");
        observations.push((
            observe(
                &configured,
                &source,
                expected(vec![finding(
                    Locale::En,
                    HelpLevel::Full,
                    Severity::Error,
                    Span::new(5, 20),
                )]),
            ),
            refusal,
        ));
    }
    println!("inline-template-original-and-late-refusals {observations:#?}");
    for (observation, refusal) in observations {
        observation.assert_refusal(refusal);
    }
}

#[test]
fn bound_names_dynamic_arguments_and_recovery_are_not_silent_clean_results() {
    let configured = configured(Locale::En, HelpLevel::Full, Severity::Error);
    let sources = [
        (
            "<div :inline-template='value'></div>",
            Refusal::UnsupportedAttribute {
                span: Span::new(5, 29),
            },
        ),
        (
            "<div :[name]='value'></div>",
            Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                span: Span::new(5, 12),
            }),
        ),
    ];
    let observations: Vec<_> = sources
        .into_iter()
        .map(|(source, refusal)| (observe(&configured, source, expected(vec![])), refusal))
        .collect();
    println!("inline-template-binding-refusals {observations:#?}");
    for (observation, refusal) in observations {
        observation.assert_refusal(refusal);
    }
    let source = "<div";
    let result = configured.lint_native_template(source, FILE);
    let legacy = configured.lint_template(source, FILE);
    println!("inline-template-recovery {result:#?}\n{legacy:#?}");
    assert_eq!(result.unwrap_err(), Refusal::Recovered { offset: 4 });
    let expected = expected(vec![
        vize_patina::LintDiagnostic {
            rule_name: "parser/template", severity: Severity::Warning,
            message: "Unexpected end of input inside a tag; inferred the missing tag close so parsing can continue.".into(),
            start: 3, end: 4, help: None, labels: vec![], fix: None,
        },
        vize_patina::LintDiagnostic {
            rule_name: "parser/template", severity: Severity::Warning,
            message: "Element is missing end tag.".into(), start: 0, end: 4,
            help: None, labels: vec![], fix: None,
        },
    ]);
    assert_eq!(full(&legacy), full(&expected));
}

#[test]
fn ordinary_sfc_and_vue_two_keep_their_distinct_owner_and_dialect_contracts() {
    let configured = configured(Locale::En, HelpLevel::Full, Severity::Error);
    let source = format!("<template>{ORIGINAL}</template>");
    let native = configured.lint_native_sfc(&source, FILE);
    let legacy = configured.lint_sfc(&source, FILE);
    println!("inline-template-sfc-boundary {native:#?}\n{legacy:#?}");
    assert_eq!(
        native.unwrap_err(),
        NativeSfcLintRefusal::UnprovidedRule { rule: RULE.into() }
    );
    assert_eq!(
        full(&legacy),
        full(&expected(vec![finding(
            Locale::En,
            HelpLevel::Full,
            Severity::Error,
            Span::new(15, 30),
        )]))
    );
    let vue_two = configured.with_vue_version(Some(VueVersion::V2));
    let native = vue_two.lint_native_template(ORIGINAL, FILE);
    let legacy = vue_two.lint_template(ORIGINAL, FILE);
    println!("inline-template-vue2 {native:#?}\n{legacy:#?}");
    assert_eq!(
        native.unwrap_err(),
        Refusal::UnsupportedVueVersion {
            requested: VueVersion::V2
        }
    );
    assert_eq!(full(&legacy), full(&expected(vec![])));
}

// A catalog name and identical metadata never confer this instance's callback.
struct Namesake;
impl Rule for Namesake {
    fn meta(&self) -> &'static RuleMeta {
        NoDeprecatedInlineTemplate.meta()
    }
    fn run_on_template<'a>(&self, context: &mut LintContext<'a>, root: &RootNode<'a>) {
        context.error("actual custom namesake", &root.loc);
    }
}

#[test]
fn custom_namesakes_and_unknown_requested_rules_have_no_builtin_capability() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(Namesake));
    let namesake = Linter::with_registry(registry);
    let native = namesake.lint_native_template(ORIGINAL, FILE);
    let legacy = namesake.lint_template(ORIGINAL, FILE);
    println!("inline-template-namesake {native:#?}\n{legacy:#?}");
    assert_eq!(
        native.unwrap_err(),
        Refusal::UnprovidedRule { rule: RULE.into() }
    );
    assert_eq!(
        full(&legacy),
        full(&expected(vec![vize_patina::LintDiagnostic {
            rule_name: RULE,
            severity: Severity::Error,
            message: "actual custom namesake".into(),
            start: 0,
            end: 0,
            help: None,
            labels: vec![],
            fix: None,
        }]))
    );
    let configured = configured(Locale::En, HelpLevel::Full, Severity::Error)
        .with_enabled_rules(Some(vec![RULE.into(), "unknown/unprovided".into()]));
    let observation = observe(
        &configured,
        ORIGINAL,
        expected(vec![finding(
            Locale::En,
            HelpLevel::Full,
            Severity::Error,
            Span::new(5, 20),
        )]),
    );
    println!("inline-template-unknown {observation:#?}");
    observation.assert_refusal(Refusal::UnprovidedRule {
        rule: "unknown/unprovided".into(),
    });
}
