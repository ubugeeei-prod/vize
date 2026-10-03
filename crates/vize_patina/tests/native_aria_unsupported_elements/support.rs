use serde_json::{Value, json};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    cstr,
    diag::{MessageLookup, PartKind, Severity, Stage, Witness},
};
use vize_l1::{
    SurfaceParseOptions, check_fidelity,
    container::{Vue, vue::DescriptorOptions},
    markup::{NativeChildren, NativeTemplateComponent},
    render,
};
use vize_patina::{
    LintResult, Linter, RuleRegistry,
    native::{NativeLintFinding, NativeSyntaxLint, header_facts::NativeHeaderFactError},
    rules::a11y::AriaUnsupportedElements,
};

pub const FILENAME: &str = "native-aria.vue";
pub const RULE: &str = "a11y/aria-unsupported-elements";
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];

pub fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
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

pub fn registered(source: &str, locale: Locale) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(AriaUnsupportedElements));
    Linter::with_registry(registry)
        .with_locale(locale)
        .lint_sfc(source, FILENAME)
}

// Compare every original parser/product diagnostic, without rule filtering.
pub fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> =
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let labels: Vec<_> = diagnostic.labels.iter().map(|label| json!({
            "message": label.message.as_str(), "start": label.start, "end": label.end,
        })).collect();
                json!({"rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
            "message": diagnostic.message.as_str(), "start": diagnostic.start,
            "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|help| help.as_str()),
            "labels": labels, "fix": diagnostic.fix})
            })
            .collect();
    json!({"filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics})
}

pub fn native(finding: &NativeLintFinding) -> Value {
    let diagnostic = finding.diagnostic();
    assert_eq!(finding.rule_name(), RULE);
    assert_eq!(diagnostic.stage, Stage::Surface);
    assert_eq!(diagnostic.severity(), Severity::Error);
    assert!(matches!(diagnostic.witness(), Some(Witness::Proven(_))));
    assert_eq!(diagnostic.witness_chain().unwrap().links().len(), 3);
    assert_eq!(diagnostic.parts.len(), 1);
    let help = &diagnostic.parts[0];
    assert_eq!(help.kind, PartKind::Help);
    assert_eq!(help.span, diagnostic.span);
    json!({"rule_name": finding.rule_name(), "severity": "error",
        "message": diagnostic.message.as_str(), "start": diagnostic.span.start,
        "end": diagnostic.span.end, "help": help.message.as_str(), "labels": [], "fix": null})
}

pub fn expected(mut diagnostics: Vec<Value>) -> Value {
    diagnostics.sort_by_key(|diagnostic| {
        (
            diagnostic["start"].as_u64().unwrap(),
            diagnostic["end"].as_u64().unwrap(),
        )
    });
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic["severity"] == "error")
        .count();
    json!({"filename": FILENAME, "error_count": errors,
        "warning_count": diagnostics.len() - errors, "diagnostics": diagnostics})
}

pub fn span(source: &str, spelling: &str) -> Span {
    let start = u32::try_from(source.find(spelling).unwrap()).unwrap();
    Span::new(start, start + u32::try_from(spelling.len()).unwrap())
}

pub fn error(locale: Locale, range: Span, tag: &str, attribute: &str) -> Value {
    json!({"rule_name": RULE, "severity": "error",
        "message": translator().format(locale, "a11y/aria-unsupported-elements.message", &[("tag", tag), ("attr", attribute)]),
        "start": range.start, "end": range.end,
        "help": translator().get(locale, "a11y/aria-unsupported-elements.help").as_ref(),
        "labels": [], "fix": null})
}

pub fn parser(severity: &str, message: &str, range: Span) -> Value {
    json!({"rule_name": "parser/template", "severity": severity,
        "message": message, "start": range.start, "end": range.end,
        "help": null, "labels": [], "fix": null})
}

// Caller traversal of real original projections; the product consumer itself
// reads SDK tables and performs no second body walk or source parse.
pub fn collect<'a>(
    lint: &NativeSyntaxLint<'_, 'a>,
    children: NativeChildren<'_, 'a>,
    messages: &impl MessageLookup,
    output: &mut Vec<NativeLintFinding>,
) -> Result<(), NativeHeaderFactError> {
    for child in children {
        if let Some(element) = child.into_element() {
            let facts = lint.header_facts(&element)?;
            let findings = facts.aria_unsupported_elements(messages).unwrap();
            for finding in &findings {
                assert_eq!(
                    facts.verify(finding.diagnostic().witness_chain().unwrap()),
                    Ok(())
                );
            }
            output.extend(findings);
            collect(lint, element.children(), messages, output)?;
        }
    }
    Ok(())
}

pub fn parity(source: &str, locale: Locale) -> Value {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let original_source = owner.component().carrier().tree.source;
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let mut findings = Vec::new();
    collect(
        &lint,
        owner.children(),
        &translator().for_locale(locale),
        &mut findings,
    )
    .unwrap();
    let actual = expected(findings.iter().map(native).collect());
    assert_eq!(
        actual,
        complete(&registered(source, locale)),
        "{source}: {locale:?}"
    );
    assert!(core::ptr::eq(lint.owner(), &owner));
    assert!(core::ptr::eq(
        original_source,
        owner.component().carrier().tree.source
    ));
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    let mut output = String::new();
    render(&owner.component().carrier().tree, &mut |piece| {
        output.push_str(piece)
    });
    assert_eq!(output, owner.component().block().source());
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    actual
}

pub struct NeverLookup;
impl MessageLookup for NeverLookup {
    fn lookup(&self, _key: &str) -> std::borrow::Cow<'static, str> {
        panic!("refused or empty genuine counterexamples cannot use a catalog")
    }
}
