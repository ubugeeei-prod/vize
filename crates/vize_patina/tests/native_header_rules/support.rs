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
    markup::{NativeChildren, NativeElement, NativeTemplateComponent},
    render,
};
use vize_patina::{
    LintResult, Linter, RuleRegistry,
    native::{NativeLintFinding, NativeLintRefusal, NativeSyntaxLint},
    rules::a11y::{NoAccessKey, NoAutofocus, NoDistractingElements},
};

pub const FILENAME: &str = "native-header.vue";
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];

#[derive(Debug, Clone, Copy)]
pub enum HeaderRule {
    Autofocus,
    AccessKey,
    Distracting,
}

impl HeaderRule {
    pub const ALL: [Self; 3] = [Self::Autofocus, Self::AccessKey, Self::Distracting];
    pub const ATTRIBUTES: [Self; 2] = [Self::Autofocus, Self::AccessKey];

    pub fn name(self) -> &'static str {
        match self {
            Self::Autofocus => "a11y/no-autofocus",
            Self::AccessKey => "a11y/no-access-key",
            Self::Distracting => "a11y/no-distracting-elements",
        }
    }

    pub fn attribute(self) -> &'static str {
        match self {
            Self::Autofocus => "autofocus",
            Self::AccessKey => "accesskey",
            Self::Distracting => panic!("the element rule has no attribute predicate"),
        }
    }

    pub fn check<'a>(
        self,
        lint: &NativeSyntaxLint<'_, 'a>,
        element: &NativeElement<'_, 'a>,
        messages: &impl MessageLookup,
    ) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
        match self {
            Self::Autofocus => lint.no_autofocus(element, messages),
            Self::AccessKey => lint.no_access_key(element, messages),
            Self::Distracting => Ok(lint
                .no_distracting_elements(element, messages)?
                .into_iter()
                .collect()),
        }
    }
}

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

pub fn registered(source: &str, locale: Locale, rule: HeaderRule) -> LintResult {
    let mut registry = RuleRegistry::new();
    match rule {
        HeaderRule::Autofocus => registry.register(Box::new(NoAutofocus)),
        HeaderRule::AccessKey => registry.register(Box::new(NoAccessKey)),
        HeaderRule::Distracting => registry.register(Box::new(NoDistractingElements)),
    }
    Linter::with_registry(registry)
        .with_locale(locale)
        .lint_sfc(source, FILENAME)
}

// Keep every parser diagnostic and every product field. These laws never
// filter the registered result down to the requested rule's diagnostics.
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

pub fn warning(rule: HeaderRule, locale: Locale, range: Span, tag: &str) -> Value {
    let message_key = cstr!("{}.message", rule.name());
    let help_key = cstr!("{}.help", rule.name());
    let message = translator().format(locale, &message_key, &[("tag", tag)]);
    json!({
        "rule_name": rule.name(), "severity": "warning", "message": message,
        "start": range.start, "end": range.end,
        "help": translator().get(locale, &help_key).as_ref(), "labels": [], "fix": null,
    })
}

pub fn parser(severity: &str, message: &str, range: Span) -> Value {
    json!({
        "rule_name": "parser/template", "severity": severity, "message": message,
        "start": range.start, "end": range.end, "help": null, "labels": [], "fix": null,
    })
}

// This is an explicit caller traversal of authentic child projections. The
// opt-in product entry inspects just the supplied element and does no walk.
fn collect<'a>(
    lint: &NativeSyntaxLint<'_, 'a>,
    children: NativeChildren<'_, 'a>,
    locale: Locale,
    rule: HeaderRule,
    output: &mut Vec<NativeLintFinding>,
) -> Result<(), NativeLintRefusal> {
    for child in children {
        if let Some(element) = child.into_element() {
            output.extend(rule.check(lint, &element, &translator().for_locale(locale))?);
            collect(lint, element.children(), locale, rule, output)?;
        }
    }
    Ok(())
}

pub fn parity(source: &str, locale: Locale, rule: HeaderRule) -> Value {
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
    collect(&lint, owner.children(), locale, rule, &mut findings).unwrap();
    let actual = expected(findings.iter().map(native).collect());
    assert_eq!(
        actual,
        complete(&registered(source, locale, rule)),
        "{rule:?}: {source}"
    );
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
