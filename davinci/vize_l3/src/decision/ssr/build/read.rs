//! Join actual original interpolation and setup rows at the existing Enter.

use super::super::{SsrSetupExpression, SsrSetupRead, SsrSetupReadKind, SsrUnsupported};
use oxc_ast::ast::Expression;
use vize_l0::id::NodeId;
use vize_l2::{
    expr::JsExpr,
    file::{DeclarationKind, InitializerKind, Namespace},
    lang::js::NativeSelectedSetup,
    resolution::Usage,
};

pub(super) fn classify<'owner, 'arena>(
    setup: &NativeSelectedSetup<'owner, 'arena>,
    node: NodeId,
    expression: &JsExpr<'arena>,
) -> Result<SsrSetupExpression<'owner, 'arena>, SsrUnsupported> {
    let file = setup.file();
    let record = file
        .native_interpolation(node)
        .ok_or(SsrUnsupported::FileExpression)?;
    let syntax = record.input().operand().syntax();
    let original = syntax
        .admitted_expression()
        .ok_or(SsrUnsupported::FileExpression)?;
    let source = syntax.source();
    if !core::ptr::eq(original.expression(), expression.ast)
        || !core::ptr::eq(source.text(), expression.source)
        || !core::ptr::eq(source.authored_root(), file.artifact().source())
        || source.span() != expression.span
        || expression.coordinates.is_none()
    {
        return Err(SsrUnsupported::FileExpression);
    }
    let resolution = file
        .expression(node)
        .ok_or(SsrUnsupported::FileExpression)?;
    let table = resolution.table().ok_or(SsrUnsupported::FileExpression)?;
    let retained = table.expression();
    let coordinates = match (retained.coordinates, expression.coordinates) {
        (Some(left), Some(right)) => core::ptr::eq(left, right),
        _ => false,
    };
    if !core::ptr::eq(retained.ast, expression.ast)
        || !core::ptr::eq(retained.source, expression.source)
        || retained.span != expression.span
        || !coordinates
    {
        return Err(SsrUnsupported::FileExpression);
    }
    // This is the real template region scope. Setup declarations separately
    // retain their original setup-root scope through setup.binding below.
    if !resolution.scope().is_some_and(|scope| {
        file.scopes()
            .get(scope.index() as usize)
            .is_some_and(|record| record.id == scope)
    }) {
        return Err(SsrUnsupported::FileScope);
    }
    let primitive = matches!(
        original.expression(),
        Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::StringLiteral(_)
    );
    let identifier = original.expression().get_identifier_reference();
    let normalized = identifier.is_some_and(|identifier| {
        matches!(table.occurrences(), [occurrence]
            if occurrence.name == identifier.name.as_str()
                && expression.ast_span_to_source(identifier.span) == Some(occurrence.span)
                && expression.source.get(occurrence.span.start as usize..occurrence.span.end as usize)
                    == Some(occurrence.name))
    });
    if original.has_legacy_literals()
        || !(primitive && table.occurrences().is_empty() || normalized)
    {
        return Err(SsrUnsupported::Expression);
    }
    // The checked grammar is a primitive (zero reads) or direct Identifier
    // (one read). Its genuine bound needs no source-sized read allocation.
    let mut read = None;
    for occurrence in table.occurrences() {
        let binding = resolution
            .binding(occurrence.binding)
            .filter(|binding| resolution.accepts(*binding))
            .ok_or(SsrUnsupported::FileBinding)?;
        let declaration = setup
            .binding(binding)
            .map_err(|_| SsrUnsupported::SetupReadAccess)?
            .declaration()
            .ok_or(SsrUnsupported::FileBinding)?;
        if declaration.namespace != Namespace::Value || declaration.name != occurrence.name {
            return Err(SsrUnsupported::FileBinding);
        }
        if occurrence.usage != Usage::Read {
            return Err(SsrUnsupported::SetupReadAccess);
        }
        let kind = match declaration.kind {
            DeclarationKind::Const
                if declaration.initializer == InitializerKind::PrimitiveLiteral =>
            {
                SsrSetupReadKind::SetupConst
            }
            DeclarationKind::Let | DeclarationKind::Var => SsrSetupReadKind::SetupLet,
            _ => return Err(SsrUnsupported::SetupReadAccess),
        };
        read = Some(SsrSetupRead {
            occurrence,
            binding,
            kind,
        });
    }
    Ok(SsrSetupExpression { resolution, read })
}
