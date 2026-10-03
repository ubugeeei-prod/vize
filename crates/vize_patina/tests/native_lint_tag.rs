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

#[test]
fn actual_registered_sfc_classification_matches_case_custom_and_opaque_is_controls() {
    for (body, kind, warnings) in [
        ("<input autofocus />", MarkupElementKind::Element, 1),
        ("<widget autofocus></widget>", MarkupElementKind::Element, 1),
        (
            "<my-element autofocus></my-element>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<marquee autofocus></marquee>",
            MarkupElementKind::Element,
            1,
        ),
        ("<blink autofocus></blink>", MarkupElementKind::Element, 1),
        (
            "<foo:bar autofocus></foo:bar>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<teleport autofocus></teleport>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<component :is='view' autofocus></component>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<input is='Foo' :is='view' autofocus />",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<div v-is='view' autofocus></div>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<Foo is='input' autofocus/>",
            MarkupElementKind::Component,
            0,
        ),
        ("<INPUT autofocus/>", MarkupElementKind::Component, 0),
        ("<Teleport autofocus/>", MarkupElementKind::Component, 0),
        ("<Suspense autofocus/>", MarkupElementKind::Component, 0),
        ("<KeepAlive autofocus/>", MarkupElementKind::Component, 0),
        (
            "<BaseTransition autofocus/>",
            MarkupElementKind::Component,
            0,
        ),
        ("<Transition autofocus/>", MarkupElementKind::Component, 0),
        (
            "<TransitionGroup autofocus/>",
            MarkupElementKind::Component,
            0,
        ),
        ("<AÉFoo autofocus/>", MarkupElementKind::Component, 0),
        ("<aÉfoo autofocus></aÉfoo>", MarkupElementKind::Element, 1),
    ] {
        let source = cstr!("<template>{body}</template>");
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            let (output, result) = parity(&source, locale);
            assert_eq!(output.len(), 1, "{body}");
            assert_eq!(output[0].kind, kind, "{body}");
            assert_eq!(result.error_count, 0, "{body}");
            assert_eq!(result.warning_count, warnings, "{body}");
        }
    }
}

#[test]
fn foreign_namespaces_keep_the_registered_component_exemptions() {
    let source = "<template><svg><g autofocus></g><foreignObject><input autofocus /><Foo autofocus/></foreignObject></svg><math><mrow autofocus></mrow><INPUT autofocus/></math></template>";
    let (output, result) = parity(source, Locale::En);
    assert_eq!(
        output
            .iter()
            .map(|item| (item.tag.as_str(), item.kind))
            .collect::<Vec<_>>(),
        [
            ("svg", MarkupElementKind::Element),
            ("g", MarkupElementKind::Element),
            ("foreignObject", MarkupElementKind::Element),
            ("input", MarkupElementKind::Element),
            ("Foo", MarkupElementKind::Component),
            ("math", MarkupElementKind::Element),
            ("mrow", MarkupElementKind::Element),
            ("INPUT", MarkupElementKind::Component),
        ]
    );
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 3);
}

#[test]
fn actual_template_and_slot_observations_keep_structural_heads_and_lookalikes() {
    for (head, kind) in [
        ("", MarkupElementKind::Element),
        ("v-if='ok'", MarkupElementKind::Template),
        ("v-else-if='ok'", MarkupElementKind::Template),
        ("v-else", MarkupElementKind::Template),
        ("v-for='item in items'", MarkupElementKind::Template),
        ("v-slot", MarkupElementKind::Template),
        ("v-slot:name", MarkupElementKind::Template),
        ("#name", MarkupElementKind::Template),
        ("v-if:arg.mod='ok'", MarkupElementKind::Template),
        ("v-iffoo='ok'", MarkupElementKind::Element),
        ("v-IF='ok'", MarkupElementKind::Element),
        (":if='ok'", MarkupElementKind::Element),
        (".slot='value'", MarkupElementKind::Element),
        ("@slot='handler'", MarkupElementKind::Element),
        ("v-slotty='value'", MarkupElementKind::Element),
    ] {
        let source = cstr!(
            "<template><template {head} autofocus></template><slot autofocus></slot><Slot autofocus/></template>"
        );
        let (output, _) = parity(&source, Locale::En);
        assert_eq!(
            output
                .iter()
                .map(|item| (item.tag.as_str(), item.kind))
                .collect::<Vec<_>>(),
            [
                ("template", kind),
                ("slot", MarkupElementKind::Slot),
                ("Slot", MarkupElementKind::Component),
            ],
            "{head}"
        );
    }
}

#[test]
fn exact_and_inherited_pre_keep_real_registered_output_and_component_exemptions() {
    let source = "<template><section v-pre autofocus><template v-if='ok' autofocus><slot autofocus></slot><Foo autofocus/></template><input :autofocus='ignored' autofocus /></section><input :autofocus='enabled' /></template>";
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let (output, result) = parity(source, locale);
        assert_eq!(
            output
                .iter()
                .map(|item| (item.tag.as_str(), item.kind))
                .collect::<Vec<_>>(),
            [
                ("section", MarkupElementKind::Element),
                ("template", MarkupElementKind::Element),
                ("slot", MarkupElementKind::Slot),
                ("Foo", MarkupElementKind::Component),
                ("input", MarkupElementKind::Element),
                ("input", MarkupElementKind::Element),
            ]
        );
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 5);
    }
    for attributes in [
        "v-pre.foo v-pre",
        "v-pre v-pre.foo",
        "v-pre:arg v-pre",
        "v-pre v-pre:arg",
    ] {
        let source = cstr!(
            "<template><template {attributes} v-if='ok' autofocus><Foo autofocus/><input :autofocus='opaque' autofocus /></template></template>"
        );
        let (output, result) = parity(&source, Locale::En);
        assert_eq!(output[0].kind, MarkupElementKind::Element);
        assert_eq!(output[1].kind, MarkupElementKind::Component);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 2);
    }
}

#[test]
fn modified_pre_refusals_preserve_the_unfiltered_registered_oracle_control() {
    for head in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        let source = cstr!(
            "<template><template {head} v-if='ok'><input :autofocus='enabled' /></template><input autofocus /></template>"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let template = owner.children().next().unwrap().into_element().unwrap();
        assert!(matches!(
            template.lint_tag(),
            Err(NativeLintTagRefusal::AmbiguousVerbatim)
        ));
        assert!(template.surface().open.is_verbatim());
        let child = template.children().next().unwrap().into_element().unwrap();
        assert!(matches!(
            child.lint_tag(),
            Err(NativeLintTagRefusal::AmbiguousVerbatim)
        ));
        assert_eq!(child.surface().open.attrs[0].name.text, ":autofocus");
        let outside = owner.children().nth(1).unwrap().into_element().unwrap();
        assert_eq!(
            outside.lint_tag().unwrap().kind(),
            NativeLintTagKind::Element
        );
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            let (output, result) = observed(&source, locale, &owner);
            assert_eq!(
                output
                    .iter()
                    .map(|item| (item.tag.as_str(), item.kind))
                    .collect::<Vec<_>>(),
                [
                    ("template", MarkupElementKind::Template),
                    ("input", MarkupElementKind::Element),
                    ("input", MarkupElementKind::Element),
                ]
            );
            assert_eq!(result.error_count, 0);
            assert_eq!(result.warning_count, 2);
        }
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
        assert!(owner.component().carrier().unsupported.is_empty());
    }
}

#[test]
fn duplicate_attribute_advisories_remain_in_the_complete_registered_output() {
    let source = "<!--🦀--><template><div autofocus autofocus></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert!(owner.component().carrier().errors.is_empty());
    let div = owner.children().next().unwrap().into_element().unwrap();
    assert_eq!(div.surface().open.attrs.len(), 2);
    assert_eq!(div.lint_tag().unwrap().kind(), NativeLintTagKind::Element);
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let (_, result) = observed(source, locale, &owner);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 3);
        assert_eq!(result.diagnostics.len(), 3);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_name == "parser/template")
        );
    }
}

#[test]
fn non_void_self_closing_control_keeps_the_registered_notice_suppression() {
    let source = "<template><div autofocus /></template>";
    let (output, result) = parity(source, Locale::En);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0].kind, MarkupElementKind::Element);
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 1);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "parser/template")
    );
}

#[test]
fn selected_prefix_offsets_and_native_reads_leave_ordinary_routes_unchanged() {
    let source = "<!--🦀--><script lang=ts>const fake='<input autofocus />'</script><template>日本語<div><input autofocus /><Foo autofocus/></div></template><script setup lang=ts>const other='<slot autofocus />'</script>";
    let before = Linter::new().lint_sfc(source, "native.vue");
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let (output, result) = parity(source, locale);
        assert_eq!(output.len(), 3);
        let start = u32::try_from(source.find("<input autofocus /><Foo").unwrap()).unwrap();
        assert_eq!(output[1].tag_span, Span::new(start + 1, start + 6));
        assert_eq!(result.warning_count, 1);
        assert_eq!(result.diagnostics[0].start, start + 7);
        assert_eq!(result.diagnostics[0].end, start + 16);
    }
    let after = Linter::new().lint_sfc(source, "native.vue");
    assert_eq!(complete_result(&before), complete_result(&after));
}

#[test]
fn equal_source_foreign_receipts_cannot_be_used_as_the_selected_lint_owner() {
    struct UnusedMessages;
    impl vize_l0::diag::MessageLookup for UnusedMessages {
        fn lookup(&self, _key: &str) -> std::borrow::Cow<'static, str> {
            panic!("foreign owner must refuse before any diagnostic lookup");
        }
    }
    let source = "<template><input autofocus /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let foreign = selected(&arena, source);
    let element = foreign.children().next().unwrap().into_element().unwrap();
    let receipt = element.lint_tag().unwrap();
    assert!(core::ptr::eq(receipt.component(), foreign.component()));
    assert!(!core::ptr::eq(receipt.component(), owner.component()));
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert!(matches!(
        lint.img_alt(&element, &UnusedMessages),
        Err(NativeLintRefusal::ForeignElement)
    ));
    assert!(matches!(
        lint.iframe_has_title(&element, &UnusedMessages),
        Err(NativeLintRefusal::ForeignElement)
    ));
    assert!(matches!(
        lint.tabindex_no_positive(&element, &UnusedMessages),
        Err(NativeLintRefusal::ForeignElement)
    ));
    assert_eq!(check_fidelity(&foreign.component().carrier().tree), Ok(()));
}
