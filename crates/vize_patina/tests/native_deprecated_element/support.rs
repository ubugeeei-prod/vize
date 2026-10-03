use serde_json::{Value, json};
use vize_carton::i18n::{Locale, translator};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    cstr,
    diag::{MessageLookup, PartKind, Stage},
};
use vize_l1::{
    SurfaceParseOptions, check_fidelity,
    container::{Vue, vue::DescriptorOptions},
    markup::{NativeChildren, NativeTemplateComponent},
    render,
};
use vize_patina::{
    LintResult, Linter, RuleRegistry,
    native::{DEPRECATED_ELEMENT_RULE, NativeLintFinding, NativeLintRefusal, NativeSyntaxLint},
    rules::html::DeprecatedElement,
};

pub const FILENAME: &str = "native-deprecated.vue";
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];

pub fn messages(locale: Locale) -> impl MessageLookup {
    translator().for_locale(locale)
}

pub fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

pub fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(arena, source, options());
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

pub fn registered(source: &str, locale: Locale) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(DeprecatedElement));
    Linter::with_registry(registry)
        .with_locale(locale)
        .lint_sfc(source, FILENAME)
}

// Every parser diagnostic and every original product field are retained.
pub fn complete(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    json!({
                        "message": label.message.as_str(), "start": label.start,
                        "end": label.end,
                    })
                })
                .collect();
            json!({
                "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(), "start": diagnostic.start,
                "end": diagnostic.end,
                "help": diagnostic.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": diagnostic.fix,
            })
        })
        .collect();
    json!({
        "filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics,
    })
}

pub fn native(finding: &NativeLintFinding) -> Value {
    let diagnostic = finding.diagnostic();
    assert_eq!(finding.rule_name(), DEPRECATED_ELEMENT_RULE);
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
    json!({
        "filename": FILENAME, "error_count": errors,
        "warning_count": diagnostics.len() - errors, "diagnostics": diagnostics,
    })
}

pub fn span(source: &str, spelling: &str) -> Span {
    let start = u32::try_from(source.find(spelling).unwrap()).unwrap();
    Span::new(start, start + u32::try_from(spelling.len()).unwrap())
}

pub fn warning(locale: Locale, range: Span, tag: &str) -> Value {
    json!({
        "rule_name": DEPRECATED_ELEMENT_RULE, "severity": "warning",
        "message": translator().format(locale, "html/deprecated-element.message", &[("tag", tag)]),
        "start": range.start, "end": range.end,
        "help": translator().get(locale, "html/deprecated-element.help").as_ref(),
        "labels": [], "fix": null,
    })
}

pub fn parser(severity: &str, message: &str, range: Span) -> Value {
    json!({
        "rule_name": "parser/template", "severity": severity, "message": message,
        "start": range.start, "end": range.end, "help": null, "labels": [], "fix": null,
    })
}

// Caller traversal uses authentic readonly children; the product checks only
// the supplied element and never walks or reparses the component body.
fn collect<'a>(
    lint: &NativeSyntaxLint<'_, 'a>,
    children: NativeChildren<'_, 'a>,
    locale: Locale,
    output: &mut Vec<NativeLintFinding>,
) -> Result<(), NativeLintRefusal> {
    for child in children {
        if let Some(element) = child.into_element() {
            output.extend(lint.deprecated_element(&element, &translator().for_locale(locale))?);
            collect(lint, element.children(), locale, output)?;
        }
    }
    Ok(())
}

pub fn parity(source: &str, locale: Locale) -> Value {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let original_source = owner.component().carrier().tree.source;
    let mut original_render = String::new();
    render(&owner.component().carrier().tree, &mut |piece| {
        original_render.push_str(piece)
    });
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let mut findings = Vec::new();
    collect(&lint, owner.children(), locale, &mut findings).unwrap();
    let actual = expected(findings.iter().map(native).collect());
    assert_eq!(actual, complete(&registered(source, locale)), "{source}");
    assert!(core::ptr::eq(lint.owner(), &owner));
    assert!(core::ptr::eq(
        original_source,
        owner.component().carrier().tree.source
    ));
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    let mut after_render = String::new();
    render(&owner.component().carrier().tree, &mut |piece| {
        after_render.push_str(piece)
    });
    assert_eq!(after_render, original_render);
    assert_eq!(after_render, owner.component().block().source());
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    actual
}

pub struct NeverLookup;
impl MessageLookup for NeverLookup {
    fn lookup(&self, _key: &str) -> std::borrow::Cow<'static, str> {
        panic!("a refused or exempt header cannot format findings")
    }
}
