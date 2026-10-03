//! Dev-only observations of the actual registered SFC rule and its full output.

use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
    cstr,
};
use vize_l1::{
    SurfaceParseOptions, check_fidelity,
    container::{Vue, vue::DescriptorOptions},
    markup::{NativeChildren, NativeLintTagKind, NativeLintTagRefusal, NativeTemplateComponent},
};
use vize_patina::{
    LintContext, LintResult, Linter, Locale, MarkupBinding, MarkupContext, MarkupElement,
    MarkupElementKind, MarkupRule, Rule, RuleMeta, RuleRegistry,
    native::{NativeLintRefusal, NativeSyntaxLint},
    rules::a11y::NoAutofocus,
};
use vize_relief::ElementNode;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    tag: String,
    kind: MarkupElementKind,
    tag_span: Span,
    opening_span: Span,
}

struct ObserveRegisteredRule {
    observations: Arc<Mutex<Vec<Observation>>>,
    // The actual selected SourceBlock supplies the same SFC offset that the
    // ordinary entry adds after linting its template. No span is guessed.
    template_offset: u32,
}

impl Rule for ObserveRegisteredRule {
    fn meta(&self) -> &'static RuleMeta {
        Rule::meta(&NoAutofocus)
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        Rule::enter_element(&NoAutofocus, ctx, element);
    }
}

impl MarkupRule for ObserveRegisteredRule {
    fn name(&self) -> &'static str {
        MarkupRule::name(&NoAutofocus)
    }

    fn enter_element<'a>(&self, _ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let opening = element.range();
        let start = self.template_offset + opening.start;
        self.observations.lock().unwrap().push(Observation {
            tag: element.tag().to_owned(),
            kind: element.kind(),
            tag_span: Span::new(
                start + 1,
                start + 1 + u32::try_from(element.tag().len()).unwrap(),
            ),
            opening_span: Span::new(start, self.template_offset + opening.end),
        });
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        MarkupRule::enter_binding(&NoAutofocus, ctx, element, binding);
    }
}

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
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

// Compare every diagnostic, including parser advisories, help, labels and fixes.
// The observer delegates the registered rule instead of replacing its behavior.
fn complete_result(result: &LintResult) -> Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
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
                "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": diagnostic.fix,
            })
        })
        .collect();
    json!({
        "filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics,
    })
}

fn registered(source: &str, locale: Locale) -> LintResult {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoAutofocus));
    Linter::with_registry(registry)
        .with_locale(locale)
        .lint_sfc(source, "native.vue")
}

fn observed(
    source: &str,
    locale: Locale,
    owner: &NativeTemplateComponent<'_>,
) -> (Vec<Observation>, LintResult) {
    let observations = Arc::new(Mutex::new(Vec::new()));
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(ObserveRegisteredRule {
        observations: Arc::clone(&observations),
        template_offset: owner.component().block().start(),
    }));
    let actual = Linter::with_registry(registry)
        .with_locale(locale)
        .lint_sfc(source, "native.vue");
    let reference = registered(source, locale);
    assert_eq!(
        complete_result(&actual),
        complete_result(&reference),
        "{source}"
    );
    let captured = observations.lock().unwrap().clone();
    (captured, actual)
}

fn native_kind(kind: NativeLintTagKind) -> MarkupElementKind {
    match kind {
        NativeLintTagKind::Element => MarkupElementKind::Element,
        NativeLintTagKind::Component => MarkupElementKind::Component,
        NativeLintTagKind::Slot => MarkupElementKind::Slot,
        NativeLintTagKind::Template => MarkupElementKind::Template,
    }
}

fn native_observations<'a>(
    owner: &NativeTemplateComponent<'a>,
    children: NativeChildren<'_, 'a>,
    output: &mut Vec<Observation>,
) {
    for child in children {
        if let Some(element) = child.into_element() {
            let receipt = element.lint_tag().unwrap();
            assert!(core::ptr::eq(receipt.component(), owner.component()));
            assert!(core::ptr::eq(receipt.element(), element.surface()));
            let block = owner.component().block();
            let start = block
                .span_of(element.surface().open.lt_name.text)
                .unwrap()
                .start;
            let end = block.span_of(element.surface().open.gt.text).unwrap().end;
            output.push(Observation {
                tag: element.surface().tag().to_owned(),
                kind: native_kind(receipt.kind()),
                tag_span: receipt.span(),
                opening_span: Span::new(start, end),
            });
            native_observations(owner, element.children(), output);
        }
    }
}

fn parity(source: &str, locale: Locale) -> (Vec<Observation>, LintResult) {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let before = cstr!("{:?}", owner.component().carrier());
    let mut native = Vec::new();
    native_observations(&owner, owner.children(), &mut native);
    let (reference, result) = observed(source, locale, &owner);
    assert_eq!(native, reference, "{source}");
    assert_eq!(cstr!("{:?}", owner.component().carrier()), before);
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    (native, result)
}

#[path = "native_lint_tag/custody.rs"]
mod custody;
#[path = "native_lint_tag/grammar.rs"]
mod grammar;
#[path = "native_lint_tag/pre.rs"]
mod pre;
