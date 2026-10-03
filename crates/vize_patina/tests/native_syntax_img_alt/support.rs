use serde_json::{Value, json};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    diag::{PartKind, Stage},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::{NativeChildren, NativeTemplateComponent},
};
use vize_patina::{
    LintDiagnostic, Linter, RuleRegistry,
    native::{NativeLintFinding, NativeSyntaxLint},
    rules::a11y::{IframeHasTitle, ImgAlt, TabindexNoPositive},
};

pub fn owner<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

pub fn native(finding: &NativeLintFinding) -> Value {
    let diagnostic = finding.diagnostic();
    assert_eq!(diagnostic.stage, Stage::Surface);
    assert_eq!(diagnostic.severity(), vize_l0::diag::Severity::Warning);
    assert!(diagnostic.witness().is_none());
    assert_eq!(diagnostic.parts.len(), 1);
    let help = &diagnostic.parts[0];
    assert_eq!(help.kind, PartKind::Help);
    assert_eq!(help.span, diagnostic.span);
    json!({
        "rule_name": finding.rule_name(), "severity": diagnostic.severity().as_str(),
        "message": diagnostic.message.as_str(), "start": diagnostic.span.start,
        "end": diagnostic.span.end, "help": help.message.as_str(),
        "labels": [], "fix": null,
    })
}

fn legacy(diagnostic: &LintDiagnostic) -> Value {
    let labels: Vec<_> = diagnostic
        .labels
        .iter()
        .map(|label| {
            json!({
                "message": label.message.as_str(), "start": label.start, "end": label.end,
            })
        })
        .collect();
    json!({
        "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
        "message": diagnostic.message.as_str(), "start": diagnostic.start,
        "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|text| text.as_str()),
        "labels": labels, "fix": diagnostic.fix,
    })
}

#[derive(Clone, Copy)]
enum SyntaxRule {
    ImgAlt,
    IframeTitle,
    Tabindex,
}

// The test caller drives original children. The product entry never walks them.
fn collect<'a>(
    lint: &NativeSyntaxLint<'_, 'a>,
    children: NativeChildren<'_, 'a>,
    locale: Locale,
    rule: SyntaxRule,
    output: &mut Vec<Value>,
) {
    for child in children {
        if let Some(element) = child.into_element() {
            let messages = translator().for_locale(locale);
            match rule {
                SyntaxRule::ImgAlt | SyntaxRule::IframeTitle => {
                    let result = match rule {
                        SyntaxRule::ImgAlt => lint.img_alt(&element, &messages),
                        _ => lint.iframe_has_title(&element, &messages),
                    };
                    if let Some(finding) = result.unwrap() {
                        output.push(native(&finding));
                    }
                }
                SyntaxRule::Tabindex => {
                    output.extend(
                        lint.tabindex_no_positive(&element, &messages)
                            .unwrap()
                            .iter()
                            .map(native),
                    );
                }
            }
            collect(lint, element.children(), locale, rule, output);
        }
    }
}

pub fn parity(source: &str, locale: Locale) -> Vec<Value> {
    compare(source, locale, SyntaxRule::ImgAlt)
}

pub fn iframe_parity(source: &str, locale: Locale) -> Vec<Value> {
    compare(source, locale, SyntaxRule::IframeTitle)
}

pub fn tabindex_parity(source: &str, locale: Locale) -> Vec<Value> {
    compare(source, locale, SyntaxRule::Tabindex)
}

pub fn tabindex_reference(source: &str, locale: Locale) -> Vec<Value> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(TabindexNoPositive));
    let result = Linter::with_registry(registry)
        .with_locale(locale)
        .lint_sfc(source, "native.vue");
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, result.diagnostics.len());
    result.diagnostics.iter().map(legacy).collect()
}

fn compare(source: &str, locale: Locale, rule: SyntaxRule) -> Vec<Value> {
    let arena = Allocator::default();
    let original = owner(&arena, source);
    let lint = NativeSyntaxLint::new(&original).unwrap();
    let before = original.component().carrier().tree.source;
    let mut actual = Vec::new();
    collect(&lint, original.children(), locale, rule, &mut actual);
    let mut registry = RuleRegistry::new();
    match rule {
        SyntaxRule::ImgAlt => registry.register(Box::new(ImgAlt)),
        SyntaxRule::IframeTitle => registry.register(Box::new(IframeHasTitle)),
        SyntaxRule::Tabindex => registry.register(Box::new(TabindexNoPositive)),
    }
    let reference = Linter::with_registry(registry)
        .with_locale(locale)
        .lint_sfc(source, "native.vue");
    assert_eq!(reference.error_count, 0);
    assert_eq!(reference.warning_count, actual.len());
    assert_eq!(
        actual,
        reference.diagnostics.iter().map(legacy).collect::<Vec<_>>(),
        "{source}"
    );
    assert!(core::ptr::eq(
        before,
        original.component().carrier().tree.source
    ));
    assert!(core::ptr::eq(lint.owner(), &original));
    actual
}
