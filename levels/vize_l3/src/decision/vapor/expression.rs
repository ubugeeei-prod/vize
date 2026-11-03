//! Original operand and immutable occurrence custody, joined only on Enter.

use super::VaporUnsupported;
use oxc_ast::ast::Expression;
use vize_l0::id::NodeId;
use vize_l2::{
    expr::{ExprRef, JsExpr},
    file::{DeclarationKind, FileResolution, InitializerKind, Namespace},
    lang::js::NativeSelectedSetup,
    op::InterpolationOp,
    resolution::Usage,
};

/// Primitive const and mutable lexical reads have distinct runtime spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporValueKind {
    Scalar,
    SetupConst,
    SetupMutable,
}

/// A genuine node's original resolution, not a caller-supplied binding list.
#[derive(Clone, Copy)]
pub struct VaporExpression<'owner, 'arena> {
    resolution: FileResolution<'owner, 'arena>,
    kind: VaporValueKind,
}
impl core::fmt::Debug for VaporExpression<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VaporExpression")
            .field("node", &self.resolution.node())
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}
impl<'owner, 'arena> VaporExpression<'owner, 'arena> {
    #[must_use]
    pub const fn resolution(&self) -> FileResolution<'owner, 'arena> {
        self.resolution
    }
    #[must_use]
    pub const fn kind(&self) -> VaporValueKind {
        self.kind
    }
}

pub(super) fn admit<'owner, 'arena>(
    setup: &NativeSelectedSetup<'owner, 'arena>,
    node: NodeId,
    interpolation: &InterpolationOp<'arena>,
) -> Result<VaporExpression<'owner, 'arena>, VaporUnsupported> {
    let file = setup.file();
    let ExprRef::Js(expression) = interpolation.expression else {
        return Err(VaporUnsupported::Expression);
    };
    let record = file
        .native_interpolation(node)
        .ok_or(VaporUnsupported::ExpressionOrigin)?;
    let operand = record.input().operand();
    let syntax = operand.syntax();
    let original = syntax
        .admitted_expression()
        .ok_or(VaporUnsupported::ExpressionOrigin)?;
    let source = syntax.source();
    if operand.full_span() != interpolation.span
        || !core::ptr::eq(original.expression(), expression.ast)
        || !core::ptr::eq(source.text(), expression.source)
        || !core::ptr::eq(source.authored_root(), file.artifact().source())
        || source.span() != expression.span
        || expression.coordinates.is_none()
    {
        return Err(VaporUnsupported::ExpressionOrigin);
    }
    let resolution = file
        .expression(node)
        .ok_or(VaporUnsupported::ExpressionOrigin)?;
    let table = resolution
        .table()
        .ok_or(VaporUnsupported::ExpressionOrigin)?;
    if !matches_expression(table.expression(), expression)
        || !resolution.scope().is_some_and(|scope| {
            file.scopes()
                .get(scope.index() as usize)
                .is_some_and(|record| record.id == scope)
        })
    {
        return Err(VaporUnsupported::ExpressionOrigin);
    }
    if matches!(original.expression(), Expression::StringLiteral(value)
        if value.lone_surrogates || value.value.as_bytes().iter().any(|byte| matches!(*byte, b'\0' | b'\r')))
    {
        return Err(VaporUnsupported::StringNormalization);
    }
    let kind = match original.expression() {
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::StringLiteral(_)
            if !original.has_legacy_literals() && table.occurrences().is_empty() =>
        {
            VaporValueKind::Scalar
        }
        Expression::Identifier(identifier) => {
            let decoded = expression
                .ast_span_to_source(identifier.span)
                .ok_or(VaporUnsupported::ExpressionOrigin)?;
            if expression
                .source
                .get(decoded.start as usize..decoded.end as usize)
                != Some(identifier.name.as_str())
            {
                return Err(VaporUnsupported::Expression);
            }
            let [occurrence] = table.occurrences() else {
                return Err(VaporUnsupported::SetupRead);
            };
            if occurrence.usage != Usage::Read
                || occurrence.name != identifier.name.as_str()
                || occurrence.span != decoded
            {
                return Err(VaporUnsupported::SetupRead);
            }
            let binding = resolution
                .binding(occurrence.binding)
                .filter(|binding| resolution.accepts(*binding))
                .ok_or(VaporUnsupported::SetupRead)?;
            let declaration = setup
                .binding(binding)
                .map_err(|_| VaporUnsupported::SetupRead)?
                .declaration()
                .ok_or(VaporUnsupported::SetupRead)?;
            if matches!(
                declaration.initializer,
                InitializerKind::PrimitiveStringWithNulOrCr
                    | InitializerKind::PrimitiveStringWithLoneSurrogates
            ) {
                return Err(VaporUnsupported::StringNormalization);
            }
            if declaration.namespace != Namespace::Value
                || declaration.name != occurrence.name
                || declaration.initializer != InitializerKind::PrimitiveLiteral
            {
                return Err(VaporUnsupported::SetupRead);
            }
            match declaration.kind {
                DeclarationKind::Const => VaporValueKind::SetupConst,
                DeclarationKind::Let | DeclarationKind::Var => VaporValueKind::SetupMutable,
                _ => return Err(VaporUnsupported::SetupRead),
            }
        }
        _ => return Err(VaporUnsupported::Expression),
    };
    Ok(VaporExpression { resolution, kind })
}

fn matches_expression(left: &JsExpr<'_>, right: &JsExpr<'_>) -> bool {
    core::ptr::eq(left.ast, right.ast)
        && core::ptr::eq(left.source, right.source)
        && left.span == right.span
        && match (left.coordinates, right.coordinates) {
            (Some(left), Some(right)) => core::ptr::eq(left, right),
            _ => false,
        }
}
