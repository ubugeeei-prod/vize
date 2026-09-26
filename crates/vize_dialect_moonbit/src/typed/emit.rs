//! Typed signatures become interface parameters, never executable bodies.

use super::TypedProjection;
use crate::projection::{Position, PositionKind, Projection, Role, SpanLink};
use vize_extension_contract::typed_expression::{Demand, TypedExpressionBatch};
use vize_l0::{Allocator, Span, String, append};
use vize_l2::expr::ForeignExpr;

pub(super) fn project<'a>(
    allocator: &'a Allocator,
    batch: &'a TypedExpressionBatch,
) -> TypedProjection<'a> {
    let mut environment = String::from("package \"vize/environment\"\n");
    let mut projection = Projection {
        file_name: "Typed.vue.mbt".into(),
        text: String::default(),
        links: Vec::new(),
        positions: Vec::new(),
        unsupported: Vec::new(),
    };
    for (index, expression) in batch.expressions.iter().enumerate() {
        let scope = batch.scope(expression);
        let signatures = scope
            .iter()
            .map(|b| b.signature.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let parameters = scope
            .iter()
            .map(|b| vize_l0::cstr!("{} : {}", b.name, b.signature))
            .collect::<Vec<_>>()
            .join(", ");
        let names = scope
            .iter()
            .map(|b| b.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let id = expression.id;
        append!(
            environment,
            "pub fn __vize_scope_{id}({signatures}) -> Unit\n"
        );
        append!(
            projection.text,
            "fn __vize_expr_{id}({parameters}) -> Unit {{\n  @environment.__vize_scope_{id}({names})\n  "
        );
        let statement_start = offset(&projection.text);
        let (prefix, suffix, kind) = match expression.expected {
            Demand::Value => ("__vize_bind(", ")", PositionKind::Bind),
            Demand::Show => ("__vize_show(", ")", PositionKind::Interpolation),
            Demand::Condition => ("__vize_cond(", ")", PositionKind::Condition),
            Demand::Name => ("__vize_name(", ")", PositionKind::Argument),
            Demand::Handler => ("__vize_on(", ")", PositionKind::Handler),
            Demand::Statement => ("__vize_on(fn() { ", " })", PositionKind::HandlerStatement),
        };
        projection.text.push_str(prefix);
        let start = offset(&projection.text);
        projection.text.push_str(&expression.source);
        projection.links.push(SpanLink {
            generated: Span::new(start, offset(&projection.text)),
            source: expression.span.into(),
            role: Role::Expression(index),
        });
        projection.text.push_str(suffix);
        let statement = Span::new(statement_start, offset(&projection.text));
        projection.text.push_str("\n}\n");
        let expr = allocator.alloc(ForeignExpr {
            dialect: crate::sfc::DIALECT,
            source: &expression.source,
            span: expression.span.into(),
            facts: vize_l0::Vec::new_in(&allocator),
        });
        projection.positions.push(Position {
            kind,
            expr,
            statement,
        });
    }
    projection.text.push_str("\nfn[T] __vize_bind(value : T) -> Unit { ignore(value) }\nfn[T : Show] __vize_show(value : T) -> Unit { ignore(value.to_string()) }\nfn __vize_cond(value : Bool) -> Unit { ignore(value) }\nfn __vize_name(value : String) -> Unit { ignore(value) }\nfn __vize_on(value : () -> Unit) -> Unit { ignore(value) }\n");
    TypedProjection {
        environment,
        projection,
    }
}

#[expect(
    clippy::expect_used,
    reason = "the host caps the entire projection below u32 capacity"
)]
fn offset(text: &str) -> u32 {
    u32::try_from(text.len()).expect("projection fits byte offsets")
}
