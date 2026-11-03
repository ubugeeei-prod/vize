//! The batch walks bindings once per element and keeps each rule's order.

use crate::ir::ByteRange;
use crate::linter::Linter;
use crate::markup::{MarkupBinding, MarkupContext, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleMeta, RuleRegistry};
use vize_l0::cstr;

struct OrderProbe(&'static RuleMeta);

impl MarkupRule for OrderProbe {
    fn name(&self) -> &'static str {
        self.0.name
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, element: &MarkupElement<'a>) {
        let range = element.range();
        ctx.lint().error_at("element", range);
    }

    fn enter_binding<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _: &MarkupElement<'a>,
        binding: &MarkupBinding<'a>,
    ) {
        let message = cstr!("binding {}", binding.arg_name().unwrap_or("?"));
        ctx.lint().error_at(message, ByteRange::new(0, 1));
    }
}

impl Rule for OrderProbe {
    fn meta(&self) -> &'static RuleMeta {
        self.0
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        Some(self)
    }
}

#[test]
fn each_rule_reports_its_bindings_beside_its_element() {
    let all = RuleRegistry::with_all();
    let meta = |name: &str| {
        all.rules()
            .iter()
            .find(|rule| rule.meta().name == name)
            .unwrap()
            .meta()
    };
    let (first, second) = (super::super::RULES[0], super::super::RULES[1]);
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(OrderProbe(meta(first))));
    registry.register(Box::new(OrderProbe(meta(second))));
    let result = Linter::with_registry(registry).lint_sfc(
        r#"<template><div id="a" :title="b"/></template>"#,
        "test.vue",
    );
    let order: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.rule_name, diagnostic.message.as_str()))
        .collect();
    assert_eq!(
        order,
        [
            (first, "element"),
            (first, "binding id"),
            (first, "binding title"),
            (second, "element"),
            (second, "binding id"),
            (second, "binding title"),
        ]
    );
}
