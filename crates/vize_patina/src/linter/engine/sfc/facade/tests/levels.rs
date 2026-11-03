//! Each rule reads the level it declares, and both levels report alike.

use crate::linter::Linter;
use crate::markup::{
    MarkupBinding, MarkupConditional, MarkupContext, MarkupDirective, MarkupDocument,
    MarkupElement, MarkupLevel, MarkupList, MarkupRule, MarkupText,
};
use crate::rule::{Rule, RuleMeta, RuleRegistry};
use std::sync::atomic::{AtomicU8, Ordering};

/// A registered rule served at a chosen level, otherwise unchanged.
struct Declared {
    rule: &'static dyn Rule,
    level: MarkupLevel,
}

impl Declared {
    fn inner(&self) -> &dyn MarkupRule {
        self.rule.as_markup_rule().unwrap()
    }
}

impl Rule for Declared {
    fn meta(&self) -> &'static RuleMeta {
        self.rule.meta()
    }
    fn as_sfc_script_rule(&self) -> Option<&dyn crate::rules::script::ScriptRule> {
        self.rule.as_sfc_script_rule()
    }
    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        self.rule.as_markup_rule().map(|_| self as &dyn MarkupRule)
    }
    fn as_native_template_rule(&self) -> Option<&dyn crate::native::template::NativeTemplateRule> {
        self.rule.as_native_template_rule()
    }
    fn as_native_sfc_template_rule(
        &self,
    ) -> Option<&dyn crate::native::sfc::NativeSfcTemplateRule> {
        self.rule.as_native_sfc_template_rule()
    }
    fn jsx_needs_lowering(&self) -> bool {
        self.rule.jsx_needs_lowering()
    }
    fn run_on_sfc<'a>(&self, ctx: &mut crate::context::LintContext<'a>) {
        self.rule.run_on_sfc(ctx);
    }
    fn run_on_template<'a>(
        &self,
        ctx: &mut crate::context::LintContext<'a>,
        root: &vize_relief::RootNode<'a>,
    ) {
        self.rule.run_on_template(ctx, root);
    }
    fn enter_element<'a>(
        &self,
        ctx: &mut crate::context::LintContext<'a>,
        element: &vize_relief::ElementNode<'a>,
    ) {
        Rule::enter_element(self.rule, ctx, element);
    }
    fn exit_element<'a>(
        &self,
        ctx: &mut crate::context::LintContext<'a>,
        element: &vize_relief::ElementNode<'a>,
    ) {
        Rule::exit_element(self.rule, ctx, element);
    }
    fn check_directive<'a>(
        &self,
        ctx: &mut crate::context::LintContext<'a>,
        element: &vize_relief::ElementNode<'a>,
        directive: &vize_relief::DirectiveNode<'a>,
    ) {
        self.rule.check_directive(ctx, element, directive);
    }
    fn check_for<'a>(
        &self,
        ctx: &mut crate::context::LintContext<'a>,
        for_node: &vize_relief::ForNode<'a>,
    ) {
        self.rule.check_for(ctx, for_node);
    }
    fn check_if<'a>(
        &self,
        ctx: &mut crate::context::LintContext<'a>,
        if_node: &vize_relief::IfNode<'a>,
    ) {
        self.rule.check_if(ctx, if_node);
    }
    fn check_interpolation<'a>(
        &self,
        ctx: &mut crate::context::LintContext<'a>,
        interpolation: &vize_relief::InterpolationNode<'a>,
    ) {
        self.rule.check_interpolation(ctx, interpolation);
    }
}

impl MarkupRule for Declared {
    fn name(&self) -> &'static str {
        self.inner().name()
    }
    fn level(&self) -> MarkupLevel {
        self.level
    }
    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        assert_eq!(
            document.lowered().is_some(),
            self.level == MarkupLevel::L2,
            "{}",
            self.name()
        );
        self.inner().enter_document(ctx, document);
    }
    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.inner().enter_element(ctx, element);
    }
    fn enter_attributes<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.inner().enter_attributes(ctx, element);
    }
    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.inner().exit_element(ctx, element);
    }
    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        self.inner().enter_binding(ctx, element, binding);
    }
    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        self.inner().enter_directive(ctx, element, directive);
    }
    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
        self.inner().enter_conditional(ctx, conditional);
    }
    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        self.inner().enter_list(ctx, list);
    }
    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        self.inner().enter_text(ctx, text);
    }
    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: crate::ir::ByteRange) {
        self.inner().enter_interpolation(ctx, range);
    }
}

fn facade_linter(level: MarkupLevel) -> Linter {
    let all: &'static RuleRegistry = Box::leak(Box::new({
        let mut all = RuleRegistry::with_all();
        all.register_opt_in_rules();
        all
    }));
    let mut registry = RuleRegistry::new();
    for rule in all.rules() {
        if super::super::RULES.contains(&rule.meta().name) {
            registry.register(Box::new(Declared {
                rule: rule.as_ref(),
                level,
            }));
        }
    }
    Linter::with_registry(registry)
}

#[test]
fn every_facade_rule_reports_alike_at_l1_and_l2() {
    let surface = facade_linter(MarkupLevel::L1);
    let lowered = facade_linter(MarkupLevel::L2);
    let mut sources: Vec<vize_l0::String> = crate::markup::differential::TEMPLATES
        .iter()
        .map(|(_, source)| vize_l0::String::from(*source))
        .collect();
    sources.extend(
        crate::markup::differential::scan_rule_fixtures()
            .templates
            .into_iter()
            .map(|(_, source)| vize_l0::String::from(source.as_str())),
    );
    // L2 inserts implicit table owners; L1 keeps the authored children.
    sources.extend(
        [
            "<table><tr><td>a</td></tr></table>",
            "<table>\n  <tr v-for=\"row in rows\" :key=\"row\"><td role=\"button\">{{ row }}</td></tr>\n</table>",
            "<table><thead><th>h</th></thead><td><img src=\"x\"></td></table>",
            "<table><template v-if=\"ok\"><tr><td><a href=\"#\">x</a></td></tr></template></table>",
        ]
        .map(vize_l0::String::from),
    );
    for source in &sources {
        let sfc = vize_l0::cstr!("<template>{source}</template>");
        let expected = lowered.lint_sfc(&sfc, "test.vue");
        let actual = surface.lint_sfc(&sfc, "test.vue");
        assert_eq!(
            vize_l0::cstr!("{:?}", actual.diagnostics),
            vize_l0::cstr!("{:?}", expected.diagnostics),
            "{source}"
        );
    }
}

struct Observe {
    meta: &'static RuleMeta,
    level: MarkupLevel,
    /// 0 before the document hook, then 1 + whether L2 was reachable.
    lowered: AtomicU8,
}

impl Rule for Observe {
    fn meta(&self) -> &'static RuleMeta {
        self.meta
    }
    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }
}

impl MarkupRule for Observe {
    fn name(&self) -> &'static str {
        self.meta.name
    }
    fn level(&self) -> MarkupLevel {
        self.level
    }
    fn enter_document(&self, _: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        self.lowered.store(
            1 + u8::from(document.lowered().is_some()),
            Ordering::Relaxed,
        );
    }
}

#[test]
fn a_rule_reaches_l2_only_when_it_declares_it() {
    for level in [MarkupLevel::L1, MarkupLevel::L2] {
        let observe = Box::leak(Box::new(Observe {
            meta: crate::rules::a11y::ImgAlt.meta(),
            level,
            lowered: AtomicU8::new(0),
        }));
        struct Shared(&'static Observe);
        impl Rule for Shared {
            fn meta(&self) -> &'static RuleMeta {
                self.0.meta
            }
            fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
                Some(self.0)
            }
        }
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(Shared(observe)));
        Linter::with_registry(registry).lint_sfc("<template><img></template>", "test.vue");
        let expected = 1 + u8::from(level == MarkupLevel::L2);
        assert_eq!(
            observe.lowered.load(Ordering::Relaxed),
            expected,
            "{level:?}"
        );
    }
}
