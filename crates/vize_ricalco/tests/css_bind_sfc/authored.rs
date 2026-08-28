//! Authored-span validation helpers for the css_bind SFC lane: every op,
//! binding, expression, diagnostic, fact, and provenance span must be a
//! valid UTF-8 range whose slice equals its authored source bytes.

use vize_ricalco::Lowered;
use vize_s0::{SourceRoot, Span};
use vize_s2::expr::ExprRef;
use vize_s2::op::{Attribute, BindingOp, DynamicName, ForBinding, Op, Region};
use vize_s2::scope::{ScopeFacts, ScopeOrigin};

pub(crate) fn assert_authored_artifact(source: &str, lowered: &Lowered<'_>) {
    let root = SourceRoot::new(source).expect("authored source");
    for op in &lowered.root.ops {
        assert_op(source, root, op);
    }
    for diagnostic in &lowered.diagnostics {
        assert_span(source, root, diagnostic.span, "diagnostic");
    }
    for record in &lowered.provenance {
        assert_span(source, root, record.span, "provenance");
    }
    for (_, facts) in lowered.scopes.iter() {
        assert_scope_facts(source, root, facts);
    }
    for (_, parts) in lowered.texts.iter() {
        for part in &parts.parts {
            assert_span(source, root, part.span, "text part");
        }
    }
    for (_, keys) in lowered.wrappers.iter() {
        for key in keys.branches.iter().flatten() {
            assert_wrapper_key(source, root, key);
        }
    }
    for (_, wrapper) in lowered.for_wrappers.iter() {
        if let Some(key) = &wrapper.key {
            assert_wrapper_key(source, root, key);
        }
    }
}

fn assert_op(source: &str, root: SourceRoot<'_>, op: &Op<'_>) {
    match op {
        Op::Element(element) => {
            assert_span(source, root, element.span, "element");
            assert_attributes(source, root, &element.attributes);
            assert_bindings(source, root, &element.bindings);
            assert_region(source, root, &element.children);
        }
        Op::Component(component) => {
            assert_span(source, root, component.span, "component");
            assert_attributes(source, root, &component.attributes);
            assert_bindings(source, root, &component.bindings);
            assert_region(source, root, &component.children);
        }
        Op::Text(text) => {
            assert_span(source, root, text.span, "text");
        }
        Op::Interpolation(interpolation) => {
            assert_span(source, root, interpolation.span, "interpolation");
            assert_expr(source, root, interpolation.expression);
        }
        Op::If(if_op) => {
            assert_span(source, root, if_op.span, "if");
            for branch in &if_op.branches {
                assert_span(source, root, branch.span, "if branch");
                if let Some(condition) = branch.condition {
                    assert_expr(source, root, condition);
                }
                assert_region(source, root, &branch.region);
            }
        }
        Op::For(for_op) => {
            assert_span(source, root, for_op.span, "for");
            assert_for_binding(source, root, for_op.binding);
            assert_region(source, root, &for_op.region);
        }
        Op::Slot(slot) => {
            assert_span(source, root, slot.span, "slot");
            assert_dynamic_name(source, root, slot.name);
            assert_attributes(source, root, &slot.attributes);
            assert_bindings(source, root, &slot.bindings);
            assert_region(source, root, &slot.fallback);
        }
    }
}

fn assert_region(source: &str, root: SourceRoot<'_>, region: &Region<'_>) {
    for op in &region.ops {
        assert_op(source, root, op);
    }
}

fn assert_attributes(source: &str, root: SourceRoot<'_>, attrs: &[Attribute<'_>]) {
    for attr in attrs {
        assert_span(source, root, attr.span, "attribute");
    }
}

fn assert_bindings(source: &str, root: SourceRoot<'_>, bindings: &[BindingOp<'_>]) {
    for binding in bindings {
        match binding {
            BindingOp::Bind(bind) => {
                assert_span(source, root, bind.span, "bind");
                if let Some(name) = bind.name {
                    assert_dynamic_name(source, root, name);
                }
                if let Some(value) = bind.value {
                    assert_expr(source, root, value);
                }
            }
            BindingOp::On(on) => {
                assert_span(source, root, on.span, "on");
                if let Some(name) = on.name {
                    assert_dynamic_name(source, root, name);
                }
                if let Some(handler) = on.handler {
                    assert_expr(source, root, handler);
                }
            }
            BindingOp::Model(model) => {
                assert_span(source, root, model.span, "model");
                assert_expr(source, root, model.contract.read);
                assert_expr(source, root, model.contract.write);
                assert_attributes(source, root, &model.attributes);
            }
            BindingOp::SlotContent(content) => {
                assert_span(source, root, content.span, "slot content");
                if let Some(name) = content.name {
                    assert_dynamic_name(source, root, name);
                }
                if let Some(params) = content.params {
                    assert_expr(source, root, params);
                }
            }
            BindingOp::VueDirective(directive) => {
                assert_span(source, root, directive.span, "vue directive");
                if let Some(argument) = directive.argument {
                    assert_dynamic_name(source, root, argument);
                }
                if let Some(value) = directive.value {
                    assert_expr(source, root, value);
                }
            }
            BindingOp::VueCssBind(css) => {
                assert_span(source, root, css.span, "css bind");
                assert_expr(source, root, css.value);
            }
            BindingOp::VueSync(sync) => {
                assert_span(source, root, sync.span, "sync");
                assert_expr(source, root, sync.value);
            }
            BindingOp::VueSlotScope(slot) => {
                assert_span(source, root, slot.span, "slot scope");
                if let Some(params) = slot.params {
                    assert_expr(source, root, params);
                }
            }
            BindingOp::VueOnce(once) => {
                assert_span(source, root, once.span, "once");
            }
            BindingOp::VueMemo(memo) => {
                assert_span(source, root, memo.span, "memo");
                assert_expr(source, root, memo.value);
            }
            BindingOp::VueShow(show) => {
                assert_span(source, root, show.span, "show");
                assert_expr(source, root, show.value);
            }
            BindingOp::VueHtml(html) => {
                assert_span(source, root, html.span, "html");
                if let Some(value) = html.value {
                    assert_expr(source, root, value);
                }
            }
        }
    }
}

fn assert_for_binding(source: &str, root: SourceRoot<'_>, binding: ForBinding<'_>) {
    assert_expr(source, root, binding.source);
    assert_expr(source, root, binding.value);
    if let Some(key) = binding.key {
        assert_expr(source, root, key);
    }
    if let Some(index) = binding.index {
        assert_expr(source, root, index);
    }
}

fn assert_dynamic_name(source: &str, root: SourceRoot<'_>, name: DynamicName<'_>) {
    if let DynamicName::Dynamic(expr) = name {
        assert_expr(source, root, expr);
    }
}

fn assert_scope_facts(source: &str, root: SourceRoot<'_>, facts: &ScopeFacts) {
    for binding in &facts.bindings {
        if let ScopeOrigin::Authored { span } = binding.origin {
            assert_span(source, root, span, "scope binding");
            assert_eq!(exact_slice(source, span), binding.name.as_str());
        }
    }
}

fn assert_wrapper_key(source: &str, root: SourceRoot<'_>, key: &vize_ricalco::lower::WrapperKey) {
    match key {
        vize_ricalco::lower::WrapperKey::Static { span, .. }
        | vize_ricalco::lower::WrapperKey::Dynamic { span, .. } => {
            assert_span(source, root, *span, "wrapper key");
        }
    }
}

fn assert_expr(source: &str, root: SourceRoot<'_>, expr: ExprRef<'_>) {
    let span = expr.span();
    assert_span(source, root, span, "expression");
    assert_eq!(exact_slice(source, span), expr.source());
}

fn assert_span(source: &str, root: SourceRoot<'_>, span: Span, label: &str) {
    assert!(
        root.contains_span(span),
        "{label} span @{}:{} is not a valid authored UTF-8 range in source length {}",
        span.start,
        span.end,
        source.len()
    );
}

fn exact_slice(source: &str, span: Span) -> &str {
    &source[span.start as usize..span.end as usize]
}
