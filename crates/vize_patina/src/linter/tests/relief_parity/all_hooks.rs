//! Run the same configured rule with every traversal hook enabled.

use crate::context::LintContext;
use crate::ir::ByteRange;
use crate::markup::{
    MarkupBinding, MarkupConditional, MarkupContext, MarkupDirective, MarkupDocument,
    MarkupElement, MarkupLevel, MarkupList, MarkupRule, MarkupText,
};
use crate::rule::{Rule, RuleMeta};

pub(super) struct AllHooks(pub(super) Box<dyn Rule>);

impl AllHooks {
    fn markup(&self) -> &dyn MarkupRule {
        self.0.as_markup_rule().unwrap()
    }
}

impl Rule for AllHooks {
    fn meta(&self) -> &'static RuleMeta {
        self.0.meta()
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }

    fn as_native_template_rule(&self) -> Option<&dyn crate::native::template::NativeTemplateRule> {
        self.0.as_native_template_rule()
    }

    fn as_native_sfc_template_rule(
        &self,
    ) -> Option<&dyn crate::native::sfc::NativeSfcTemplateRule> {
        self.0.as_native_sfc_template_rule()
    }

    fn jsx_needs_lowering(&self) -> bool {
        self.0.jsx_needs_lowering()
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        self.0.run_on_sfc(ctx);
    }
}

impl MarkupRule for AllHooks {
    fn name(&self) -> &'static str {
        self.markup().name()
    }

    fn level(&self) -> MarkupLevel {
        self.markup().level()
    }

    // Deliberately keep the default Full traversal. Forward every callback so
    // a missing traversal declaration cannot silently discard a future hook.
    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        self.markup().enter_document(ctx, document);
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.markup().enter_element(ctx, element);
    }

    fn enter_attributes<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.markup().enter_attributes(ctx, element);
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        self.markup().exit_element(ctx, element);
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        self.markup().enter_binding(ctx, element, binding);
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        self.markup().enter_directive(ctx, element, directive);
    }

    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
        self.markup().enter_conditional(ctx, conditional);
    }

    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        self.markup().enter_list(ctx, list);
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        self.markup().enter_text(ctx, text);
    }

    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
        self.markup().enter_interpolation(ctx, range);
    }
}
