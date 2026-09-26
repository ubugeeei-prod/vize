//! Forward each hook in registry order during one L2 walk.

use crate::ir::ByteRange;
use crate::markup::{
    MarkupConditional, MarkupContext, MarkupDirective, MarkupDocument, MarkupElement, MarkupList,
    MarkupRule, MarkupText,
};

pub(super) struct Rules<'a>(pub &'a [(usize, &'static str, &'a dyn MarkupRule)]);

impl MarkupRule for Rules<'_> {
    fn name(&self) -> &'static str {
        ""
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        for (_, name, rule) in self.0 {
            if *name == super::TEMPLATE_DOCUMENT_RULE {
                continue;
            }
            ctx.lint().current_rule = name;
            rule.enter_document(ctx, document);
        }
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.enter_element(ctx, element);
            // The admitted binding checks originate in legacy element hooks.
            // Keep each rule's attribute reports beside its element reports,
            // before the next rule can report direct child text.
            element.walk_bindings(&mut |binding| {
                ctx.lint().current_rule = name;
                rule.enter_binding(ctx, element, &binding);
            });
        }
    }

    fn enter_attributes<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.enter_attributes(ctx, element);
        }
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.exit_element(ctx, element);
        }
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    ) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.enter_directive(ctx, element, directive);
        }
    }

    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.enter_conditional(ctx, conditional);
        }
    }

    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.enter_list(ctx, list);
        }
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.enter_text(ctx, text);
        }
    }

    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
        for (_, name, rule) in self.0 {
            ctx.lint().current_rule = name;
            rule.enter_interpolation(ctx, range);
        }
    }
}
