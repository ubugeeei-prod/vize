use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use vize_l0::{Span, String, ToCompactString, cstr};
use vize_l1::markup::{DirectiveName, NativeLintComponent};
use vize_patina::{
    HelpLevel, LintContext, LintResult, Linter, Locale, Rule, RuleCategory, RuleMeta, RuleRegistry,
    Severity,
    native::template::{
        NativeTemplateAttributeKind as Kind, NativeTemplateAttributeProfile as Profile,
        NativeTemplateElement, NativeTemplateLintContext, NativeTemplateLintRefusal as Refusal,
        NativeTemplateRule,
    },
};
use vize_relief::{ElementNode, PropNode, RootNode};

pub const MESSAGE: &str = "vue/component-definition-name-casing.message";
pub const HELP: &str = "vue/component-definition-name-casing.help";
pub const LOCALES: [Locale; 3] = [Locale::En, Locale::Ja, Locale::Zh];
pub static FIRST: RuleMeta = RuleMeta {
    name: "test/native-element-first",
    description: "Actual first test instance",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};
pub static SECOND: RuleMeta = RuleMeta {
    name: "test/native-element-second",
    description: "Actual second test instance",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};
mod trace;
pub use trace::{Event, binding_event, element_event, static_event};
pub type Events = Arc<Mutex<Vec<Event>>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeObservation {
    pub ordinal: usize,
    pub name: String,
    pub value: Option<String>,
    pub range: Span,
    pub head: Option<DirectiveName>,
    pub binding: &'static str,
    pub argument: Option<Span>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ElementObservation {
    pub marker: &'static str,
    pub tag: String,
    pub ordinal: usize,
    pub parent: Option<String>,
    pub attributes: Vec<AttributeObservation>,
}
pub type Observations = Arc<Mutex<Vec<ElementObservation>>>;

pub struct Audit {
    pub meta: &'static RuleMeta,
    pub marker: &'static str,
    pub profile: Profile,
    pub events: Events,
    pub observations: Observations,
    pub profile_calls: Arc<AtomicUsize>,
    pub widen: Option<Arc<AtomicBool>>,
    pub fail: bool,
}
impl Audit {
    pub fn new(
        meta: &'static RuleMeta,
        marker: &'static str,
        profile: Profile,
        events: Events,
    ) -> Self {
        Self {
            meta,
            marker,
            profile,
            events,
            observations: Arc::new(Mutex::new(Vec::new())),
            profile_calls: Arc::new(AtomicUsize::new(0)),
            widen: None,
            fail: false,
        }
    }
}
impl Rule for Audit {
    fn meta(&self) -> &'static RuleMeta {
        self.meta
    }
    fn as_native_template_rule(&self) -> Option<&dyn NativeTemplateRule> {
        Some(self)
    }
    fn run_on_template<'a>(&self, context: &mut LintContext<'a>, root: &RootNode<'a>) {
        let marker = cstr!("{}/root", self.marker);
        context.warn_with_help(
            context.t_fmt(MESSAGE, &[("name", &marker)]),
            &root.loc,
            context.t(HELP),
        );
    }
    fn enter_element<'a>(&self, context: &mut LintContext<'a>, element: &ElementNode<'a>) {
        for prop in &element.props {
            let loc = match prop {
                PropNode::Attribute(attribute) => &attribute.loc,
                PropNode::Directive(directive) => &directive.loc,
            };
            context.warn_with_help(
                context.t_fmt(MESSAGE, &[("name", self.marker)]),
                loc,
                context.t(HELP),
            );
        }
    }
}
impl NativeTemplateRule for Audit {
    fn attribute_profile(&self) -> Profile {
        self.profile_calls.fetch_add(1, Ordering::SeqCst);
        if self
            .widen
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::SeqCst))
        {
            Profile::Bindings
        } else {
            self.profile
        }
    }
    fn run_on_template<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        root: &NativeLintComponent<'a>,
    ) -> Result<(), Refusal> {
        assert!(core::ptr::eq(context.owner(), root));
        self.events.lock().unwrap().push(Event::Root(self.marker));
        if let Some(flag) = &self.widen {
            flag.store(true, Ordering::SeqCst);
        }
        let marker = cstr!("{}/root", self.marker);
        context.warn_root_with_help(MESSAGE, &[("name", &marker)], HELP);
        Ok(())
    }
    fn run_on_element<'a>(
        &self,
        context: &mut NativeTemplateLintContext<'_, 'a>,
        element: &NativeTemplateElement<'_, 'a>,
    ) -> Result<(), Refusal> {
        assert!(core::ptr::eq(
            element.original().component(),
            context.owner().component()
        ));
        self.events.lock().unwrap().push(element_event(
            self.marker,
            element.original().surface().tag(),
            element.original().ordinal(),
            element
                .original()
                .parent_element()
                .map(|parent| parent.tag()),
        ));
        let mut observed = Vec::new();
        for attribute in element.attributes() {
            let original = attribute.original();
            assert!(core::ptr::eq(
                original.component(),
                context.owner().component()
            ));
            assert!(core::ptr::eq(
                original.element(),
                element.original().surface()
            ));
            assert!(core::ptr::eq(
                original.surface(),
                &original.element().open.attrs[original.ordinal()]
            ));
            let span = attribute.range();
            assert_eq!(
                &context.source()
                    [span.start as usize..span.start as usize + original.surface().name.text.len()],
                original.surface().name.text
            );
            let (binding, argument) = match attribute.kind() {
                Kind::Static { name, value } => {
                    assert_eq!(name, original.surface().name.text);
                    assert_eq!(value, attribute.value());
                    ("static", None)
                }
                Kind::Bind {
                    name,
                    argument_range,
                } => {
                    assert_eq!(
                        name,
                        &context.source()
                            [argument_range.start as usize..argument_range.end as usize]
                    );
                    ("bind", Some(argument_range))
                }
                Kind::DynamicBind { argument_range } => ("dynamic-bind", Some(argument_range)),
            };
            let observation = AttributeObservation {
                ordinal: original.ordinal(),
                name: original.surface().name.text.into(),
                value: attribute.value().map(ToCompactString::to_compact_string),
                range: span,
                head: attribute.directive(),
                binding,
                argument,
            };
            self.events.lock().unwrap().push(Event::Attribute {
                marker: self.marker,
                observation: observation.clone(),
            });
            observed.push(observation);
            context.warn_attribute_with_help(attribute, MESSAGE, &[("name", self.marker)], HELP)?;
            if self.fail {
                return Err(Refusal::UnsupportedAttribute { span });
            }
        }
        self.observations.lock().unwrap().push(ElementObservation {
            marker: self.marker,
            tag: element.original().surface().tag().into(),
            ordinal: element.original().ordinal(),
            parent: element
                .original()
                .parent_element()
                .map(|parent| parent.tag().into()),
            attributes: observed,
        });
        Ok(())
    }
}

pub fn events() -> Events {
    Arc::new(Mutex::new(Vec::new()))
}
pub fn linter(rules: Vec<Audit>, locale: Locale, help: HelpLevel) -> Linter {
    let mut registry = RuleRegistry::new();
    for rule in rules {
        registry.register(Box::new(rule));
    }
    Linter::with_registry(registry)
        .with_locale(locale)
        .with_help_level(help)
}
pub fn span(source: &str, text: &str) -> Span {
    let start = source.find(text).unwrap() as u32;
    Span::new(start, start + text.len() as u32)
}
pub fn complete(result: &LintResult) -> serde_json::Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    serde_json::json!({
                        "message": label.message.as_str(), "start": label.start, "end": label.end,
                    })
                })
                .collect();
            serde_json::json!({
                "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(), "start": diagnostic.start,
                "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": diagnostic.fix,
            })
        })
        .collect();
    serde_json::json!({ "filename": result.filename.as_str(), "diagnostics": diagnostics, "error_count": result.error_count, "warning_count": result.warning_count })
}
pub fn pair(linter: &Linter, source: &str, file: &str) -> LintResult {
    let original = linter.lint_template(source, file);
    let native = linter.lint_native_template(source, file).unwrap();
    assert_eq!(complete(&native), complete(&original), "{source}");
    assert_eq!(cstr!("{native:#?}"), cstr!("{original:#?}"));
    let repeat = linter.lint_native_template(source, file).unwrap();
    assert_eq!(cstr!("{repeat:#?}"), cstr!("{native:#?}"));
    native
}

/// Expected full configured metadata comes from the independently registered
/// generic rule's original callbacks on a fixed, valid control. Only authored
/// ranges vary below; this is a dev oracle, not a historical output recapture.
pub fn original_single(linter: &Linter, source: &str, file: &str, attributes: &[&str]) {
    assert_eq!(
        complete(&linter.lint_template(source, file)),
        single_expected(linter, source, file, attributes)
    );
}

pub fn single_expected(
    linter: &Linter,
    source: &str,
    file: &str,
    attributes: &[&str],
) -> serde_json::Value {
    let control = linter.lint_template("<div data-control='text'/>", file);
    assert_eq!(control.diagnostics.len(), 2);
    let control = complete(&control);
    let mut diagnostics = vec![control["diagnostics"][0].clone()];
    for text in attributes {
        let range = span(source, text);
        let mut diagnostic = control["diagnostics"][1].clone();
        diagnostic["start"] = range.start.into();
        diagnostic["end"] = range.end.into();
        diagnostics.push(diagnostic);
    }
    serde_json::json!({
        "filename": file, "diagnostics": diagnostics, "error_count": 0,
        "warning_count": attributes.len() + 1,
    })
}
