//! Authored-span validation for L1->L2 artifacts: every op, binding,
//! expression, diagnostic, fact, and provenance span must stay within the
//! authored UTF-8 source it claims to describe.

use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span, expression_guard::expression_is_safe_to_parse};
use vize_l1::embed::prepare_attribute_value;
use vize_l1_to_l2::Lowered;
use vize_l1_to_l2::lower::TextPart;
use vize_l2::expr::{ExprRef, OpaqueReason};
use vize_l2::op::{Attribute, BindingOp, DynamicName, ForBinding, Op, Region};
use vize_l2::scope::{ScopeFacts, ScopeOrigin};

pub fn assert_authored_artifact(source: &str, lowered: &Lowered<'_>) {
    let root = SourceRoot::new(source).expect("authored source");
    for op in &lowered.root.ops {
        assert_op(source, root, op);
    }
    for diagnostic in &lowered.diagnostics {
        assert_span(source, root, diagnostic.span, "diagnostic");
    }
    for record in &lowered.provenance {
        assert_provenance(source, root, record);
    }
    for (_, facts) in lowered.scopes.iter() {
        assert_scope_facts(source, root, facts);
    }
    for (_, parts) in lowered.texts.iter() {
        for part in &parts.parts {
            assert_text_part(source, root, part);
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
        for attr in &wrapper.attributes {
            assert_span(source, root, attr.span, "for wrapper attr");
        }
        if let Some(class) = &wrapper.class {
            assert_span(source, root, class.span, "for wrapper class");
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
        Op::Text(text) => assert_span(source, root, text.span, "text"),
        Op::Comment(comment) => assert_span(source, root, comment.span, "comment"),
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
        Op::OriginalFor(for_op) => {
            assert_span(source, root, for_op.span, "original for");
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
                if let Some(handler) = on.expression() {
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
            BindingOp::SlotsObject(object) => {
                assert_span(source, root, object.span, "slots object");
                assert_expr(source, root, object.value);
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
            BindingOp::VueOnce(once) => assert_span(source, root, once.span, "once"),
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
            BindingOp::VueText(text) => {
                assert_span(source, root, text.span, "text");
                if let Some(value) = text.value {
                    assert_expr(source, root, value);
                }
            }
            BindingOp::VueCloak(cloak) => assert_span(source, root, cloak.span, "cloak"),
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
            if exact_slice(source, span) != binding.name.as_str() {
                assert_original_identifier(source, span, binding.name.as_str());
            }
        }
    }
}

fn assert_original_identifier(source: &str, span: Span, cooked: &str) {
    let arena = Allocator::new();
    let original = prepare_attribute_value(&arena, source, span).expect("original binding token");
    assert!(expression_is_safe_to_parse(original.text()));
    let parameters = Parser::new(arena.as_oxc(), original.text(), SourceType::ts())
        .parse_v_for_parameters()
        .expect("original complete BindingIdentifier parameter goal");
    let [parameter] = parameters.items.as_slice() else {
        panic!("one original BindingIdentifier declaration");
    };
    assert!(parameters.rest.is_none());
    assert!(parameter.initializer.is_none());
    let BindingPattern::BindingIdentifier(identifier) = &parameter.pattern else {
        panic!("the whole origin is an original BindingIdentifier token");
    };
    let complete = oxc_span::Span::new(0, original.text().len() as u32);
    assert_eq!(parameters.span, complete);
    assert_eq!(parameter.span, complete);
    assert_eq!(identifier.span, complete);
    assert_eq!(identifier.name.as_str(), cooked);
    assert_eq!(
        original
            .authored_span(Span::new(identifier.span.start, identifier.span.end))
            .expect("complete authored identifier token"),
        span
    );
}

fn assert_wrapper_key(source: &str, root: SourceRoot<'_>, key: &vize_l1_to_l2::lower::WrapperKey) {
    match key {
        vize_l1_to_l2::lower::WrapperKey::Static { span, .. }
        | vize_l1_to_l2::lower::WrapperKey::Dynamic { span, .. } => {
            assert_span(source, root, *span, "wrapper key");
        }
    }
}

fn assert_expr(source: &str, root: SourceRoot<'_>, expr: ExprRef<'_>) {
    let span = expr.span();
    assert_span(source, root, span, "expression");
    let slice = exact_slice(source, span);
    let expr_source = expr.source();
    match expr {
        ExprRef::Opaque(opaque) if opaque.reason == OpaqueReason::Compound => {
            // Compound sources are rebuilt from text parts, so exact authored
            // slices are checked on each dynamic TextPart instead.
        }
        _ if slice.len() == expr_source.len() => {
            assert_eq!(
                slice, expr_source,
                "expression span is not authored when source length matches"
            );
        }
        _ => {}
    }
}

fn assert_text_part(source: &str, root: SourceRoot<'_>, part: &TextPart) {
    assert_span(source, root, part.span, "text part");
    if part.dynamic {
        let slice = exact_slice(source, part.span);
        let Some(inner) = slice
            .strip_prefix("{{")
            .and_then(|slice| slice.strip_suffix("}}"))
        else {
            panic!("dynamic text part span must cover the authored interpolation token: {slice:?}");
        };
        assert_eq!(
            inner.trim(),
            part.text.as_str(),
            "dynamic text part text must match the trimmed authored interpolation"
        );
    }
}

fn assert_provenance(
    source: &str,
    root: SourceRoot<'_>,
    record: &vize_l2::provenance::ProvenanceRecord,
) {
    assert_span(source, root, record.span, "provenance");
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

#[test]
fn exceptional_native_scope_origins_pass_the_complete_authored_artifact_law() {
    let source =
        r#"<li v-for="({ label: \u005frow, name: &#x3b1; }, \u006bey, \u0069ndex) in rows">x</li>"#;
    let arena = Allocator::new();
    let (tree, errors) = vize_l1::markup::parse(&arena, source);
    assert!(errors.is_empty());
    let lowered = vize_l1_to_l2::lower(&arena, &tree, &errors);
    assert_authored_artifact(source, &lowered);
    let (_, scope) = lowered
        .scopes
        .iter()
        .next()
        .expect("original iteration scope");
    assert_eq!(
        scope
            .bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<Vec<_>>(),
        ["_row", "α", "key", "index"]
    );
}

#[test]
fn different_scope_spellings_require_one_complete_original_identifier_token() {
    use vize_l0::String;
    use vize_l2::scope::{ScopeBinding, ScopeTag};

    for (source, cooked) in [
        (r"\u0061 + other", "a"),
        (r"\u0061/*tail*/", "a"),
        (r"{ name: \u0061 }", "a"),
        (r"\u0061, other", "a"),
        (r"\u0061 = fallback", "a"),
        (r"\u0061: Type", "a"),
        (r"\u0061", "wrong"),
    ] {
        let root = SourceRoot::new(source).unwrap();
        let facts = ScopeFacts {
            tag: ScopeTag::from_index(0),
            bindings: vec![ScopeBinding {
                name: String::from(cooked),
                origin: ScopeOrigin::Authored {
                    span: Span::new(0, source.len() as u32),
                },
            }],
        };
        assert!(
            std::panic::catch_unwind(|| assert_scope_facts(source, root, &facts)).is_err(),
            "refuse non-token origin {source:?} for cooked name {cooked:?}"
        );
    }
}
