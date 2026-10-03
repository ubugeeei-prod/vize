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
    Linter, RuleRegistry,
    diagnostic::LintDiagnostic,
    native::{NativeLintFinding, NativeSyntaxLint},
    rules::a11y::ImgAlt,
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

// The test caller drives original children. The product entry never walks them.
fn collect<'a>(
    lint: &NativeSyntaxLint<'_, 'a>,
    children: NativeChildren<'_, 'a>,
    locale: Locale,
    output: &mut Vec<Value>,
) {
    for child in children {
        if let Some(element) = child.into_element() {
            if let Some(finding) = lint
                .img_alt(&element, &translator().for_locale(locale))
                .unwrap()
            {
                output.push(native(&finding));
            }
            collect(lint, element.children(), locale, output);
        }
    }
}

pub fn parity(source: &str, locale: Locale) -> Vec<Value> {
    let arena = Allocator::default();
    let original = owner(&arena, source);
    let lint = NativeSyntaxLint::new(&original).unwrap();
    let before = original.component().carrier().tree.source;
    let mut actual = Vec::new();
    collect(&lint, original.children(), locale, &mut actual);
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ImgAlt));
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
