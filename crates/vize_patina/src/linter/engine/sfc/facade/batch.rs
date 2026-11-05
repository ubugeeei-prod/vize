//! Forward each hook in registry order during one L2 walk.

use crate::context::CallbackSite;
use crate::ir::ByteRange;
use crate::markup::{
    MarkupBinding, MarkupConditional, MarkupContext, MarkupDirective, MarkupDocument,
    MarkupElement, MarkupList, MarkupRule, MarkupText,
};
use vize_l0::SmallVec;

pub(super) struct Rules<'a, const ORDERED: bool>(
    pub &'a [(usize, &'static str, &'a dyn MarkupRule)],
);

impl<const ORDERED: bool> MarkupRule for Rules<'_, ORDERED> {
    fn name(&self) -> &'static str {
        ""
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
        if ORDERED {
            ctx.lint().replay_callback_site(CallbackSite::Template);
        }
        for (index, name, rule) in self.0 {
            if *name == super::TEMPLATE_DOCUMENT_RULE {
                continue;
            }
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_document(ctx, document);
        }
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ORDERED {
            ctx.lint()
                .replay_callback_site(CallbackSite::Element(element.range().start));
        }
        // One binding walk per element, not per rule. Each rule still reports
        // its attributes beside its element, before the next rule runs.
        let mut bindings: SmallVec<[MarkupBinding<'a>; 8]> = SmallVec::new();
        element.walk_bindings(&mut |binding| bindings.push(binding));
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_element(ctx, element);
            for binding in &bindings {
                if ORDERED {
                    ctx.lint().set_callback_rule(*index);
                }
                ctx.lint().current_rule = name;
                rule.enter_binding(ctx, element, binding);
            }
        }
    }

    fn enter_attributes<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ORDERED {
            ctx.lint()
                .replay_callback_site(CallbackSite::Attributes(element.range().start));
        }
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_attributes(ctx, element);
        }
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        if ORDERED {
            ctx.lint()
                .replay_callback_site(CallbackSite::ExitElement(element.range().start));
        }
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
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
        if ORDERED {
            ctx.lint().replay_callback_site(CallbackSite::Directive(
                element.range().start,
                directive.range().start,
            ));
        }
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_directive(ctx, element, directive);
        }
    }

    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    ) {
        if ORDERED {
            ctx.lint()
                .replay_callback_site(CallbackSite::Conditional(conditional.range().start));
        }
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_conditional(ctx, conditional);
        }
    }

    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>) {
        if ORDERED {
            ctx.lint()
                .replay_callback_site(CallbackSite::List(list.range().start));
        }
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_list(ctx, list);
        }
    }

    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>) {
        if ORDERED {
            ctx.lint()
                .replay_callback_site(CallbackSite::Text(text.range().start));
        }
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_text(ctx, text);
        }
    }

    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
        if ORDERED {
            ctx.lint()
                .replay_callback_site(CallbackSite::Interpolation(range.start));
        }
        for (index, name, rule) in self.0 {
            if ORDERED {
                ctx.lint().set_callback_rule(*index);
            }
            ctx.lint().current_rule = name;
            rule.enter_interpolation(ctx, range);
        }
    }
}
