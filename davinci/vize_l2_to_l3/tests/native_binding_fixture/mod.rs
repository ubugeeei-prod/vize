//! Rejection fixtures with real retained OXC formals; no lexical-site claim.
use oxc_ast::ast::Expression;
use vize_l0::{Allocator, Box, Span, Vec};
use vize_l2::{
    binding::{BindingRef, JsBinding},
    expr::{ExprRef, JsExpr, js::JsCoordinates},
    op::{ForBinding, ForOp, Op, Region},
};

pub const SOURCE: &str = "(x)=>items";
pub struct Fixture<'a> {
    pub binding: BindingRef<'a>,
    pub root: Region<'a>,
}

pub fn native(allocator: &Allocator) -> Fixture<'_> {
    let expression =
        JsExpr::parse_in(allocator, SOURCE, Span::new(0, SOURCE.len() as u32)).unwrap();
    let Expression::ArrowFunctionExpression(arrow) = expression.ast else {
        panic!("actual arrow fixture root")
    };
    let parameter = arrow.params.items.first().unwrap();
    let span = Span::new(parameter.span.start, parameter.span.end);
    let source = SOURCE.get(span.start as usize..span.end as usize).unwrap();
    let coordinates = allocator
        .alloc(JsCoordinates::checked(SOURCE, source, span, parameter.span.start, &[]).unwrap());
    let binding = BindingRef::Js(
        JsBinding::from_retained_in(allocator, parameter, source, span, coordinates).unwrap(),
    );
    root(allocator, binding)
}

pub fn compatibility(allocator: &Allocator) -> Fixture<'_> {
    root(
        allocator,
        ExprRef::parse_js_in(allocator, "x", Span::new(1, 2)).into(),
    )
}

fn root<'a>(allocator: &'a Allocator, binding: BindingRef<'a>) -> Fixture<'a> {
    let mut ops = Vec::new_in(&allocator);
    ops.push(Op::For(Box::new_in(
        ForOp {
            binding: ForBinding {
                source: ExprRef::parse_js_in(allocator, "items", Span::new(5, 10)),
                value: binding,
                key: None,
                index: None,
            },
            region: Region {
                ops: Vec::new_in(&allocator),
            },
            span: Span::new(0, SOURCE.len() as u32),
        },
        &allocator,
    )));
    Fixture {
        binding,
        root: Region { ops },
    }
}
