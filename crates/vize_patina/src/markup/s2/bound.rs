//! [`S2Bound`]: one attached S2 binding op, with the authored S1 attribute it
//! was lowered from when the artifact carries a surface tree.
//!
//! The `vue.*` presence and value ops (`vue.show`, `vue.cloak`, …) keep only
//! what realization reads; an authored argument or modifier on them
//! (`v-cloak.foo`) survives only on S1, where the facade reads it back.

use super::binding::op_span;
use super::surface::SurfaceDirective;
use vize_s0::Span;
use vize_s2::op::{BindingOp, DynamicName};

/// One attached binding op and its authored spelling.
#[derive(Clone, Copy)]
pub(in crate::markup) struct S2Bound<'a> {
    pub(in crate::markup) op: &'a BindingOp<'a>,
    pub(in crate::markup) surface: Option<&'a vize_s1::Attribute<'a>>,
    /// The artifact's source, where every authored slice lives.
    pub(in crate::markup) source: &'a str,
}

fn dynamic_name<'a>(name: Option<&DynamicName<'a>>) -> Option<(&'a str, bool)> {
    match name? {
        DynamicName::Static(text) => Some((text, true)),
        DynamicName::Dynamic(expression) => Some((expression.source(), false)),
    }
}

impl<'a> S2Bound<'a> {
    pub(in crate::markup) fn span(self) -> Span {
        op_span(self.op)
    }

    fn spelling(self) -> Option<SurfaceDirective<'a>> {
        SurfaceDirective::parse(self.surface?.name.text)
    }

    /// The directive name the op spells, without the `v-` prefix.
    pub(in crate::markup) fn name(self) -> &'a str {
        match self.op {
            BindingOp::Bind(_) | BindingOp::VueSync(_) | BindingOp::VueCssBind(_) => "bind",
            BindingOp::On(_) => "on",
            BindingOp::Model(_) => "model",
            BindingOp::SlotContent(_) => "slot",
            BindingOp::VueDirective(directive) => directive.name,
            BindingOp::VueSlotScope(_) => "slot-scope",
            BindingOp::VueOnce(_) => "once",
            BindingOp::VueMemo(_) => "memo",
            BindingOp::VueShow(_) => "show",
            BindingOp::VueHtml(_) => "html",
            BindingOp::VueText(_) => "text",
            BindingOp::VueCloak(_) => "cloak",
        }
    }

    /// The argument (`click` for `@click`, the bracket contents for a dynamic
    /// argument) and whether it is static.
    pub(in crate::markup) fn arg(self) -> Option<(&'a str, bool)> {
        match self.op {
            BindingOp::Bind(bind) => dynamic_name(bind.name.as_ref()),
            BindingOp::On(on) => dynamic_name(on.name.as_ref()),
            BindingOp::Model(model) => dynamic_name(model.argument.as_ref()),
            BindingOp::SlotContent(content) => dynamic_name(content.name.as_ref()),
            BindingOp::VueDirective(directive) => dynamic_name(directive.argument.as_ref()),
            BindingOp::VueSync(sync) => Some((sync.name, true)),
            BindingOp::VueCssBind(_)
            | BindingOp::VueSlotScope(_)
            | BindingOp::VueOnce(_)
            | BindingOp::VueMemo(_)
            | BindingOp::VueShow(_)
            | BindingOp::VueHtml(_)
            | BindingOp::VueText(_)
            | BindingOp::VueCloak(_) => {
                let spelling = self.spelling()?;
                spelling.arg.map(|arg| (arg, spelling.arg_static))
            }
        }
    }

    /// The authored argument's range, when the op carries an authored
    /// argument slice (read off S1 when the artifact has a surface).
    pub(in crate::markup) fn arg_range(self) -> Option<crate::ir::ByteRange> {
        let arg = match self.spelling() {
            Some(spelling) => spelling.arg?,
            None => self.arg()?.0,
        };
        super::surface::slice_range(self.source, arg).or_else(|| {
            // A JSX projection carries the argument as a name, not a source
            // slice; it is authored inside the op's own span.
            let span = self.span();
            let text = self.source.get(span.start as usize..span.end as usize)?;
            let at = span.start + u32::try_from(text.find(arg)?).ok()?;
            Some(crate::ir::ByteRange::new(at, at + arg.len() as u32))
        })
    }

    /// Visit the authored modifiers. `ui.model` carries its dialect modifiers
    /// as attributes after the synthesized `element-kind` attribute.
    pub(in crate::markup) fn walk_modifiers(self, visitor: &mut impl FnMut(&'a str)) {
        let modifiers: &'a [&'a str] = match self.op {
            BindingOp::Bind(bind) => &bind.modifiers,
            BindingOp::On(on) => &on.modifiers,
            BindingOp::SlotContent(content) => &content.modifiers,
            BindingOp::VueDirective(directive) => &directive.modifiers,
            BindingOp::Model(model) => {
                for attribute in model.attributes.iter() {
                    if attribute.name != "element-kind" {
                        visitor(attribute.name);
                    }
                }
                return;
            }
            BindingOp::VueSync(sync) => {
                visitor("sync");
                &sync.modifiers
            }
            BindingOp::VueCssBind(_)
            | BindingOp::VueSlotScope(_)
            | BindingOp::VueOnce(_)
            | BindingOp::VueMemo(_)
            | BindingOp::VueShow(_)
            | BindingOp::VueHtml(_)
            | BindingOp::VueText(_)
            | BindingOp::VueCloak(_) => {
                if let Some(spelling) = self.spelling() {
                    spelling.walk_modifiers(visitor);
                }
                return;
            }
        };
        for modifier in modifiers {
            visitor(modifier);
        }
    }

    /// The authored value expression, trimmed.
    pub(in crate::markup) fn expression(self) -> Option<&'a str> {
        let expression = match self.op {
            BindingOp::Bind(bind) => bind.value.as_ref(),
            BindingOp::On(on) => on.handler.as_ref(),
            BindingOp::Model(model) => Some(&model.contract.read),
            BindingOp::SlotContent(content) => content.params.as_ref(),
            BindingOp::VueDirective(directive) => directive.value.as_ref(),
            BindingOp::VueSync(sync) => Some(&sync.value),
            BindingOp::VueSlotScope(scope) => scope.params.as_ref(),
            BindingOp::VueMemo(memo) => Some(&memo.value),
            BindingOp::VueShow(show) => Some(&show.value),
            BindingOp::VueHtml(html) => html.value.as_ref(),
            BindingOp::VueText(text) => text.value.as_ref(),
            BindingOp::VueCssBind(bind) => Some(&bind.value),
            BindingOp::VueOnce(_) | BindingOp::VueCloak(_) => {
                let value = self.surface?.value.as_ref()?.content.text.trim();
                return Some(value).filter(|text| !text.is_empty());
            }
        }?;
        Some(expression.source())
    }
}
