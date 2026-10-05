//! Binding observations retain the unchanged, separate admission and Doc gates.

use super::{
    assert_output,
    failure_support::{document, prefix},
    options,
};
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, NativeSfcObservation, NativeTemplateRefusal,
    observe_native_sfc_in,
};
use vize_l0::{Allocator, Span};
use vize_l1::embed::{Grammar, Lang, Shape, syntax::EmbedHole};
use vize_l1::markup::NativeAttributeBindingExpression;

fn current(owner: &NativeSfcObservation<'_>, source: &str, raw: &str, value: Span) {
    let operand = &owner.binding_operands()[1];
    assert_eq!(operand.name_span(), Span::new(51, 54));
    assert_eq!(operand.argument_span(), Span::new(52, 54));
    assert_eq!(operand.value_span(), value);
    assert_eq!(operand.raw_value(), raw);
    assert_eq!(value.slice(source), raw);
    let syntax = operand.syntax();
    assert_eq!(syntax.source().text(), raw);
    assert_eq!(syntax.source().span(), value);
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert!(syntax.source().decode_map().is_none());
    assert_eq!(
        syntax.grammar(),
        Grammar {
            shape: Shape::Expr,
            lang: Lang::Js
        }
    );
    assert!(syntax.source_type().is_module());
    assert!(!syntax.source_type().is_typescript());
    assert_eq!(syntax.parser_prefix(), 2);
    assert_eq!(syntax.comments().count(), 0);
    assert_eq!(syntax.diagnostics().count(), 0);
    assert!(owner.binding_failure().is_none());
}

fn original_unary_leaf(operand: &NativeAttributeBindingExpression<'_>, depth: u32) {
    let syntax = operand.syntax();
    let root = syntax.expression().unwrap();
    assert_eq!(syntax.hole(), None);
    assert_eq!(root.span(), oxc_span::Span::new(2, depth + 4));
    let mut leaf = root;
    for index in 0..depth {
        let Expression::UnaryExpression(unary) = leaf else {
            panic!("original unary chain")
        };
        assert_eq!(unary.operator.as_str(), "!");
        assert_eq!(unary.span, oxc_span::Span::new(2 + index, depth + 4));
        assert_eq!(
            syntax.decoded_span(unary.span),
            Ok(Span::new(index, depth + 2))
        );
        assert_eq!(
            syntax.authored_span(unary.span),
            Ok(Span::new(56 + index, 58 + depth))
        );
        leaf = &unary.argument;
    }
    let Expression::BigIntLiteral(bigint) = leaf else {
        panic!("original BigInt leaf")
    };
    assert_eq!(bigint.value.as_str(), "1");
    assert_eq!(bigint.raw.as_ref().unwrap().as_str(), "1n");
    assert_eq!(
        syntax.decoded_span(leaf.span()),
        Ok(Span::new(depth, depth + 2))
    );
    assert_eq!(
        syntax.authored_span(leaf.span()),
        Ok(Span::new(56 + depth, 58 + depth))
    );
    assert!(core::ptr::eq(root, syntax.expression().unwrap()));
}

#[test]
fn original_depth_sixteen_binding_formats_the_complete_mixed_sfc_and_preserves_its_leaf() {
    let raw = format!("{}1n", "!".repeat(16));
    let flat = format!("{}1n", "! ".repeat(16));
    let source = format!(
        "<template><i :title='first'>{{{{1n}}}}</i><p v-if='ok' :id='{raw}'>{{{{later}}}}</p></template>"
    );
    let expected = format!(
        "<template><i :title='first'>{{{{ 1n }}}}</i><p v-if='ok' :id='{flat}'>{{{{ later }}}}</p></template>"
    );
    let arena = Allocator::default();
    let owner = assert_output(&arena, &source, &expected, options(200, 2, LineEnding::Lf));
    assert_eq!(owner.binding_operands().len(), 2);
    assert_eq!(owner.attribute_operands().len(), 1);
    assert_eq!(owner.operands().len(), 2);
    current(&owner, &source, &raw, Span::new(56, 74));
    original_unary_leaf(&owner.binding_operands()[1], 16);
}

#[test]
fn original_depth_seventeen_binding_refuses_doc_after_parking_its_admitted_current_owner() {
    let raw = format!("{}1n", "!".repeat(17));
    let source = format!(
        "<template><i :title='first'>{{{{1n}}}}</i><p v-if='ok' :id='{raw}'>{{{{later}}}}</p></template>"
    );
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, &source, options(200, 2, LineEnding::Lf));
    prefix(
        &owner,
        &source,
        document(NativeTemplateRefusal::BindingExpression {
            span: Span::new(56, 75),
            refusal: ExpressionRefusal::DepthLimit {
                span: Span::new(17, 19),
            },
        }),
        2,
        Span::new(77, 86),
    );
    current(&owner, &source, &raw, Span::new(56, 75));
    let operand = &owner.binding_operands()[1];
    original_unary_leaf(operand, 17);
    let selected = owner.selected().unwrap();
    let element = selected.children().nth(1).unwrap().into_element().unwrap();
    let view = operand
        .admitted_for(selected, element.attributes().nth(1).unwrap())
        .unwrap();
    assert!(core::ptr::eq(
        view.expression().unwrap().expression(),
        operand.syntax().expression().unwrap()
    ));
}

#[test]
fn original_wrapped_unit_admission_precedes_nesting_and_doc_depth_for_complete_binding_value() {
    let raw = format!("{}a{}", "(".repeat(32), ")".repeat(32));
    let source = format!(
        "<template><i :title='first'>{{{{1n}}}}</i><p v-if='ok' :id='{raw}'>{{{{later}}}}</p></template>"
    );
    let arena = Allocator::default();
    let owner = observe_native_sfc_in(&arena, &source, options(200, 2, LineEnding::Lf));
    // The complete parser input exceeds the existing 31-unit limit before the
    // shared nesting guard or AST Doc walk can run. No depth limit is changed.
    prefix(
        &owner,
        &source,
        document(NativeTemplateRefusal::BindingRejected {
            span: Span::new(56, 121),
            index: 1,
            hole: Some(EmbedHole::TokenBudget),
        }),
        2,
        Span::new(123, 132),
    );
    current(&owner, &source, &raw, Span::new(56, 121));
    let operand = &owner.binding_operands()[1];
    assert_eq!(operand.syntax().hole(), Some(EmbedHole::TokenBudget));
    assert!(operand.syntax().expression().is_none());
    assert!(operand.syntax().admitted_expression().is_none());
    let selected = owner.selected().unwrap();
    let element = selected.children().nth(1).unwrap().into_element().unwrap();
    assert!(
        operand
            .admitted_for(selected, element.attributes().nth(1).unwrap())
            .is_none()
    );
}
