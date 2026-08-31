//! Owned folio span oracle: every span that survives into the lifetime-free
//! S2 folio must resolve against the authored source.

use vize_s0::{SourceRoot, Span};
use vize_s2::folio::{
    DisegnoFolio, FolioAttribute, FolioBinding, FolioExpr, FolioForBinding, FolioName, FolioOp,
};

pub fn assert_folio_spans_resolve(source: &str, folio: &DisegnoFolio, context: &str) {
    let root = SourceRoot::new(source).expect("authored source");
    for op in &folio.ops {
        assert_op(source, root, op, context);
    }
}

fn assert_op(source: &str, root: SourceRoot<'_>, op: &FolioOp, context: &str) {
    match op {
        FolioOp::Element(element) => {
            assert_span(source, root, element.span, "element", context);
            assert_attributes(source, root, &element.attributes, context);
            assert_bindings(source, root, &element.bindings, context);
            assert_ops(source, root, &element.children, context);
        }
        FolioOp::Component(component) => {
            assert_span(source, root, component.span, "component", context);
            assert_attributes(source, root, &component.attributes, context);
            assert_bindings(source, root, &component.bindings, context);
            assert_ops(source, root, &component.children, context);
        }
        FolioOp::Text(text) => assert_span(source, root, text.span, "text", context),
        FolioOp::Interpolation(interpolation) => {
            assert_span(source, root, interpolation.span, "interpolation", context);
            assert_expr(source, root, &interpolation.expression, context);
        }
        FolioOp::If(if_op) => {
            assert_span(source, root, if_op.span, "if", context);
            for branch in &if_op.branches {
                assert_span(source, root, branch.span, "if branch", context);
                if let Some(condition) = &branch.condition {
                    assert_expr(source, root, condition, context);
                }
                assert_ops(source, root, &branch.ops, context);
            }
        }
        FolioOp::For(for_op) => {
            assert_span(source, root, for_op.span, "for", context);
            assert_for_binding(source, root, &for_op.binding, context);
            assert_ops(source, root, &for_op.ops, context);
        }
        FolioOp::Slot(slot) => {
            assert_span(source, root, slot.span, "slot", context);
            assert_name(source, root, &slot.name, context);
            assert_attributes(source, root, &slot.attributes, context);
            assert_bindings(source, root, &slot.bindings, context);
            assert_ops(source, root, &slot.fallback, context);
        }
    }
}

fn assert_ops(source: &str, root: SourceRoot<'_>, ops: &[FolioOp], context: &str) {
    for op in ops {
        assert_op(source, root, op, context);
    }
}

fn assert_attributes(source: &str, root: SourceRoot<'_>, attrs: &[FolioAttribute], context: &str) {
    for attr in attrs {
        assert_span(source, root, attr.span, "attribute", context);
    }
}

fn assert_bindings(source: &str, root: SourceRoot<'_>, bindings: &[FolioBinding], context: &str) {
    for binding in bindings {
        match binding {
            FolioBinding::Bind(bind) => {
                assert_span(source, root, bind.span, "bind", context);
                if let Some(name) = &bind.name {
                    assert_name(source, root, name, context);
                }
                if let Some(value) = &bind.value {
                    assert_expr(source, root, value, context);
                }
            }
            FolioBinding::On(on) => {
                assert_span(source, root, on.span, "on", context);
                if let Some(name) = &on.name {
                    assert_name(source, root, name, context);
                }
                if let Some(handler) = &on.handler {
                    assert_expr(source, root, handler, context);
                }
            }
            FolioBinding::Model(model) => {
                assert_span(source, root, model.span, "model", context);
                assert_expr(source, root, &model.contract.read, context);
                assert_expr(source, root, &model.contract.write, context);
                if let Some(argument) = &model.argument {
                    assert_name(source, root, argument, context);
                }
                assert_attributes(source, root, &model.attributes, context);
            }
            FolioBinding::SlotContent(content) => {
                assert_span(source, root, content.span, "slot content", context);
                if let Some(name) = &content.name {
                    assert_name(source, root, name, context);
                }
                if let Some(params) = &content.params {
                    assert_expr(source, root, params, context);
                }
            }
            FolioBinding::VueDirective(directive) => {
                assert_span(source, root, directive.span, "vue directive", context);
                if let Some(argument) = &directive.argument {
                    assert_name(source, root, argument, context);
                }
                if let Some(value) = &directive.value {
                    assert_expr(source, root, value, context);
                }
            }
            FolioBinding::VueCssBind(css) => {
                assert_span(source, root, css.span, "css bind", context);
                assert_expr(source, root, &css.value, context);
            }
            FolioBinding::VueSync(sync) => {
                assert_span(source, root, sync.span, "sync", context);
                assert_expr(source, root, &sync.value, context);
            }
            FolioBinding::VueSlotScope(slot) => {
                assert_span(source, root, slot.span, "slot scope", context);
                if let Some(params) = &slot.params {
                    assert_expr(source, root, params, context);
                }
            }
            FolioBinding::VueOnce(once) => assert_span(source, root, once.span, "once", context),
            FolioBinding::VueMemo(memo) => {
                assert_span(source, root, memo.span, "memo", context);
                assert_expr(source, root, &memo.value, context);
            }
            FolioBinding::VueShow(show) => {
                assert_span(source, root, show.span, "show", context);
                assert_expr(source, root, &show.value, context);
            }
            FolioBinding::VueHtml(html) => {
                assert_span(source, root, html.span, "html", context);
                if let Some(value) = &html.value {
                    assert_expr(source, root, value, context);
                }
            }
            FolioBinding::VueText(text) => {
                assert_span(source, root, text.span, "text", context);
                if let Some(value) = &text.value {
                    assert_expr(source, root, value, context);
                }
            }
            FolioBinding::VueCloak(cloak) => {
                assert_span(source, root, cloak.span, "cloak", context);
            }
        }
    }
}

fn assert_name(source: &str, root: SourceRoot<'_>, name: &FolioName, context: &str) {
    match name {
        FolioName::Static(_) => {}
        FolioName::Dynamic(expr) => assert_expr(source, root, expr, context),
    }
}

fn assert_for_binding(
    source: &str,
    root: SourceRoot<'_>,
    binding: &FolioForBinding,
    context: &str,
) {
    assert_expr(source, root, &binding.source, context);
    assert_expr(source, root, &binding.value, context);
    if let Some(key) = &binding.key {
        assert_expr(source, root, key, context);
    }
    if let Some(index) = &binding.index {
        assert_expr(source, root, index, context);
    }
}

fn assert_expr(source: &str, root: SourceRoot<'_>, expr: &FolioExpr, context: &str) {
    match expr {
        FolioExpr::Js { span, .. }
        | FolioExpr::Foreign { span, .. }
        | FolioExpr::Opaque { span, .. }
        | FolioExpr::Filter { span, .. } => {
            assert_span(source, root, *span, "expression", context);
        }
    }
}

fn assert_span(source: &str, root: SourceRoot<'_>, span: Span, label: &str, context: &str) {
    assert!(
        root.contains_span(span),
        "{label} owned-folio span @{}:{} is not a valid authored UTF-8 range in source length {}: {context}",
        span.start,
        span.end,
        source.len()
    );
}
