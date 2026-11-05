//! Full packets from real legacy and facade callbacks with identical spans.

use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::linter::{LintResult, Linter};
use crate::markup::{MarkupContext, MarkupDirective, MarkupDocument, MarkupElement, MarkupRule};
use crate::rule::{Rule, RuleMeta, RuleRegistry};
use crate::rules::{
    a11y::AltText,
    opinionated::vue::{HtmlSelfClosing, VBindStyle},
};
use vize_relief::{DirectiveNode, ElementNode, RootNode};

const SOURCE: &str = r#"<template><button v-bind:title="title"><span v-bind:title="title"></span></button></template>"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Element,
    Directive,
    Multiple,
}

fn metadata(kind: Kind) -> &'static RuleMeta {
    match kind {
        Kind::Element => Rule::meta(&HtmlSelfClosing::default()),
        Kind::Directive => Rule::meta(&VBindStyle::default()),
        Kind::Multiple => Rule::meta(&AltText),
    }
}

struct CallbackRule {
    kind: Kind,
    facade: bool,
}

fn emit(ctx: &mut LintContext<'_>, message: &'static str) {
    // Every phase and node reports the exact same span. Only real callback
    // visits can recover the authored order; range/name sorting cannot.
    ctx.report(LintDiagnostic::warn(ctx.current_rule, message, 0, 1));
}

impl Rule for CallbackRule {
    fn meta(&self) -> &'static RuleMeta {
        metadata(self.kind)
    }

    fn as_markup_rule(&self) -> Option<&dyn MarkupRule> {
        (self.facade && self.kind == Kind::Multiple).then_some(self)
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _: &RootNode<'a>) {
        if self.kind == Kind::Multiple {
            emit(ctx, "template");
        }
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, _: &ElementNode<'a>) {
        match self.kind {
            Kind::Element => emit(ctx, "element"),
            Kind::Multiple => {
                emit(ctx, "first");
                emit(ctx, "second");
            }
            Kind::Directive => {}
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        _: &ElementNode<'a>,
        _: &DirectiveNode<'a>,
    ) {
        if self.kind != Kind::Element {
            emit(ctx, "directive");
        }
    }

    fn exit_element<'a>(&self, ctx: &mut LintContext<'a>, _: &ElementNode<'a>) {
        if self.kind == Kind::Multiple {
            emit(ctx, "exit");
        }
    }
}

impl MarkupRule for CallbackRule {
    fn name(&self) -> &'static str {
        metadata(self.kind).name
    }

    fn enter_document(&self, ctx: &mut MarkupContext<'_, '_>, _: &MarkupDocument) {
        emit(ctx.lint(), "template");
    }

    fn enter_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, _: &MarkupElement<'a>) {
        emit(ctx.lint(), "first");
        emit(ctx.lint(), "second");
    }

    fn enter_directive<'a>(
        &self,
        ctx: &mut MarkupContext<'_, 'a>,
        _: &MarkupElement<'a>,
        _: &MarkupDirective<'a>,
    ) {
        emit(ctx.lint(), "directive");
    }

    fn exit_element<'a>(&self, ctx: &mut MarkupContext<'_, 'a>, _: &MarkupElement<'a>) {
        emit(ctx.lint(), "exit");
    }
}

fn linter(kinds: &[Kind], facade: bool, ordered: bool) -> Linter {
    let mut registry = RuleRegistry::new();
    for kind in kinds {
        registry.register(Box::new(CallbackRule {
            kind: *kind,
            facade,
        }));
    }
    let linter = Linter::with_registry(registry);
    if ordered {
        linter.with_editor_callback_order()
    } else {
        linter
    }
}

fn expected(events: &[(Kind, &'static str)]) -> LintResult {
    let start = "<template>".len() as u32;
    LintResult {
        filename: "order.vue".into(),
        diagnostics: events
            .iter()
            .map(|(kind, message)| {
                LintDiagnostic::warn(metadata(*kind).name, *message, start, start + 1)
            })
            .collect(),
        error_count: 0,
        warning_count: events.len(),
    }
}

fn packet(result: &LintResult) -> vize_l0::CompactString {
    vize_l0::cstr!("{result:?}")
}

#[test]
fn editor_callback_order_preserves_real_phases_configured_rank_and_nested_exit() {
    use Kind::{Directive as D, Element as E, Multiple as M};
    for (kinds, events) in [
        (
            vec![D, E, M],
            vec![
                (M, "template"),
                (E, "element"),
                (M, "first"),
                (M, "second"),
                (D, "directive"),
                (M, "directive"),
                (E, "element"),
                (M, "first"),
                (M, "second"),
                (D, "directive"),
                (M, "directive"),
                (M, "exit"),
                (M, "exit"),
            ],
        ),
        (
            vec![M, D, E],
            vec![
                (M, "template"),
                (M, "first"),
                (M, "second"),
                (E, "element"),
                (M, "directive"),
                (D, "directive"),
                (M, "first"),
                (M, "second"),
                (E, "element"),
                (M, "directive"),
                (D, "directive"),
                (M, "exit"),
                (M, "exit"),
            ],
        ),
    ] {
        let expected = packet(&expected(&events));
        let legacy = linter(&kinds, false, false).lint_sfc(SOURCE, "order.vue");
        assert_eq!(
            packet(&legacy),
            expected,
            "authored original callback packet"
        );
        let ordered_legacy = linter(&kinds, false, true).lint_sfc(SOURCE, "order.vue");
        assert_eq!(
            packet(&ordered_legacy),
            expected,
            "legacy bytes stay intact"
        );
        let facade = linter(&kinds, true, true).lint_sfc(SOURCE, "order.vue");
        assert_eq!(packet(&facade), expected, "one rule reports in four phases");
        let ordinary = linter(&kinds, true, false).lint_sfc(SOURCE, "order.vue");
        assert_ne!(
            packet(&ordinary),
            expected,
            "control exposes deferred dispatch"
        );
    }
}

#[test]
fn editor_callback_order_retains_all_unsuppressed_complete_diagnostics() {
    use Kind::{Directive as D, Element as E, Multiple as M};
    let events = [
        (M, "template"),
        (M, "first"),
        (M, "second"),
        (D, "directive"),
        (M, "directive"),
        (M, "first"),
        (M, "second"),
        (D, "directive"),
        (M, "directive"),
        (M, "exit"),
        (M, "exit"),
    ];
    for facade in [false, true] {
        let result = linter(&[D, E, M], facade, true)
            .with_disabled_rules(vec![metadata(E).name.into()])
            .lint_sfc(SOURCE, "order.vue");
        assert_eq!(packet(&result), packet(&expected(&events)));
    }
}
