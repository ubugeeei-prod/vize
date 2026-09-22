//! Hook dispatch: the rules one traversal drives — a single rule, or a fused
//! rule set that fires every subscribed rule's hook at each node in one walk
//! (the way the template visitor dispatches its rules), so a lint pass walks
//! the document once rather than once per rule.

use super::binding::MarkupBinding;
use super::directive::MarkupDirective;
use super::element::MarkupElement;
use super::hooks::{HOOK_COUNT, MarkupHooks};
use super::node::MarkupText;
use super::scope::{MarkupConditional, MarkupList};
use super::{MarkupContext, MarkupDocument, MarkupRule};
use crate::ir::ByteRange;

mod layout;

pub use layout::MarkupRuleLayout;

/// The rules a [`super::MarkupDocumentVisitor`] drives. Each hook sets the
/// reporting rule on the lint context before calling into it, and calls only
/// the rules subscribed to it ([`MarkupRule::hooks`]).
pub trait MarkupRules {
    /// Whether any driven rule subscribes to `hook`; the visitor skips the
    /// walk behind a hook nobody listens to.
    fn subscribes(&self, hook: MarkupHooks) -> bool;
    /// Fire `enter_document`.
    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument);
    /// Fire `enter_element`.
    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>);
    /// Fire `exit_element`.
    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>);
    /// Fire `enter_binding`.
    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    );
    /// Fire `enter_directive`.
    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        element: &MarkupElement<'a>,
        directive: &MarkupDirective<'a>,
    );
    /// Fire `enter_conditional`.
    fn enter_conditional<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        conditional: &MarkupConditional<'a>,
    );
    /// Fire `enter_list`.
    fn enter_list<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, list: &MarkupList<'a>);
    /// Fire `enter_text`.
    fn enter_text<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, text: &MarkupText<'a>);
    /// Fire `enter_interpolation`.
    fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange);
}

/// One rule, with its name and subscription read once.
pub(super) struct One<'r, R: ?Sized> {
    rule: &'r R,
    name: &'static str,
    hooks: MarkupHooks,
}

impl<'r, R: MarkupRule + ?Sized> One<'r, R> {
    pub(super) fn new(rule: &'r R) -> Self {
        Self {
            rule,
            name: rule.name(),
            hooks: rule.hooks(),
        }
    }
}

/// A fused rule set: per hook, the subscribed rules in registration order,
/// with their names resolved once — the dispatch table a lint pass builds
/// before it walks.
pub struct MarkupRuleSet<'r> {
    entries: Vec<(&'r dyn MarkupRule, &'static str)>,
    /// `entries[bounds[h]..bounds[h + 1]]` subscribe to hook `h`.
    bounds: [u16; HOOK_COUNT + 1],
}

impl<'r> MarkupRuleSet<'r> {
    /// Build the table for `rules`, keeping their order within each hook.
    pub fn new(rules: impl IntoIterator<Item = &'r dyn MarkupRule>) -> Self {
        let rules: Vec<&'r dyn MarkupRule> = rules.into_iter().collect();
        let mut entries = Vec::with_capacity(rules.len());
        let mut bounds = [0u16; HOOK_COUNT + 1];
        for (index, hook) in MarkupHooks::EACH.into_iter().enumerate() {
            for &rule in &rules {
                if rule.hooks().contains(hook) {
                    entries.push((rule, rule.name()));
                }
            }
            bounds[index + 1] = entries.len() as u16;
        }
        Self { entries, bounds }
    }

    /// Whether the set drives no rule at all.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    fn at(&self, hook: MarkupHooks) -> &[(&'r dyn MarkupRule, &'static str)] {
        let index = hook.index();
        &self.entries[usize::from(self.bounds[index])..usize::from(self.bounds[index + 1])]
    }
}

macro_rules! dispatch {
    ($($hook:ident [$flag:ident] ( $($arg:ident : $ty:ty),* );)*) => {
        impl<R: MarkupRule + ?Sized> MarkupRules for One<'_, R> {
            #[inline]
            fn subscribes(&self, hook: MarkupHooks) -> bool {
                self.hooks.contains(hook)
            }

            $(
                #[inline]
                fn $hook<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, $($arg: $ty),*) {
                    if self.hooks.contains(MarkupHooks::$flag) {
                        ctx.lint.current_rule = self.name;
                        self.rule.$hook(ctx, $($arg),*);
                    }
                }
            )*

            #[inline]
            fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
                if self.hooks.contains(MarkupHooks::DOCUMENT) {
                    ctx.lint.current_rule = self.name;
                    self.rule.enter_document(ctx, document);
                }
            }

            #[inline]
            fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
                if self.hooks.contains(MarkupHooks::INTERPOLATION) {
                    ctx.lint.current_rule = self.name;
                    self.rule.enter_interpolation(ctx, range);
                }
            }
        }

        impl MarkupRules for MarkupRuleSet<'_> {
            #[inline]
            fn subscribes(&self, hook: MarkupHooks) -> bool {
                !self.at(hook).is_empty()
            }

            $(
                #[inline]
                fn $hook<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, $($arg: $ty),*) {
                    for &(rule, name) in self.at(MarkupHooks::$flag) {
                        ctx.lint.current_rule = name;
                        rule.$hook(ctx, $($arg),*);
                    }
                }
            )*

            #[inline]
            fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, document: &MarkupDocument) {
                for &(rule, name) in self.at(MarkupHooks::DOCUMENT) {
                    ctx.lint.current_rule = name;
                    rule.enter_document(ctx, document);
                }
            }

            #[inline]
            fn enter_interpolation(&self, ctx: &mut MarkupContext<'_, '_>, range: ByteRange) {
                for &(rule, name) in self.at(MarkupHooks::INTERPOLATION) {
                    ctx.lint.current_rule = name;
                    rule.enter_interpolation(ctx, range);
                }
            }
        }
    };
}

dispatch! {
    enter_element[ELEMENT](element: &MarkupElement<'a>);
    exit_element[EXIT_ELEMENT](element: &MarkupElement<'a>);
    enter_binding[BINDING](element: &MarkupElement<'a>, binding: &MarkupBinding<'a>);
    enter_directive[DIRECTIVE](element: &MarkupElement<'a>, directive: &MarkupDirective<'a>);
    enter_conditional[CONDITIONAL](conditional: &MarkupConditional<'a>);
    enter_list[LIST](list: &MarkupList<'a>);
    enter_text[TEXT](text: &MarkupText<'a>);
}
