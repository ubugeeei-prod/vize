//! Existing normally attached rows are the only query authority.

use super::{
    HandlerLocalRef, L2Ir, L2IrForHead, L2IrHandler, Origin, TemplateQueryError as Error,
    TemplateSiteRef, TemplateSiteScope, TemplateSymbolRef,
};
use crate::op::{HandlerId, OriginalForId};
use crate::resolution::{HandlerBindingRef, HandlerLocalId, HandlerScopeId};
use vize_l0::Span;

pub(super) fn visit<'f, 'a>(
    l2ir: &'f L2Ir<'a>,
    mut visit: impl FnMut(TemplateSiteRef<'f, 'a>),
) -> Result<(), Error> {
    for (node, _) in l2ir.facts.handlers.iter() {
        let handler = l2ir
            .handler(HandlerId(node))
            .ok_or(Error::OriginalMembership)?;
        handlers(handler, &mut visit)?;
    }
    for (node, record) in l2ir.facts.for_heads.iter() {
        // An interrupted body can retain aliases before attaching its actual op.
        // Incomplete File refusal precedes this scan; never silently skip a row.
        if record.original.is_none() {
            return Err(Error::OriginalMembership);
        }
        let head = l2ir
            .for_head(OriginalForId(node))
            .ok_or(Error::OriginalMembership)?;
        for_head(head, &mut visit)?;
    }
    Ok(())
}

fn handlers<'f, 'a>(
    handler: L2IrHandler<'f, 'a>,
    visit: &mut impl FnMut(TemplateSiteRef<'f, 'a>),
) -> Result<(), Error> {
    let resolution = handler.resolution().ok_or(Error::OriginalMembership)?;
    let syntax = resolution.input().operand().syntax();
    if !core::ptr::eq(
        syntax.source().authored_root(),
        handler.l2ir().artifact().source(),
    ) || !syntax
        .body()
        .is_some_and(|body| core::ptr::eq(body, resolution.input().body()))
    {
        return Err(Error::OriginalMembership);
    }
    for row in resolution.declarations() {
        let span = resolution
            .authored_span(row.span)
            .map_err(|_| Error::Projection)?;
        checked_span(handler.l2ir(), span)?;
        check_scope(handler, row.scope)?;
        visit(TemplateSiteRef {
            symbol: TemplateSymbolRef::HandlerLocal(local(handler, row.binding)?),
            span,
            scope: TemplateSiteScope::Handler(handler, row.scope),
            usage: None,
            origin: Origin::Handler(handler),
        });
    }
    for row in resolution.references() {
        let span = resolution
            .authored_span(row.span)
            .map_err(|_| Error::Projection)?;
        checked_span(handler.l2ir(), span)?;
        check_scope(handler, row.scope)?;
        let symbol = match row.binding {
            HandlerBindingRef::Local(id) => TemplateSymbolRef::HandlerLocal(local(handler, id)?),
            HandlerBindingRef::Outer(id) => TemplateSymbolRef::File(
                handler
                    .l2ir()
                    .binding(id)
                    .ok_or(Error::OriginalMembership)?,
            ),
        };
        visit(TemplateSiteRef {
            symbol,
            span,
            scope: TemplateSiteScope::Handler(handler, row.scope),
            usage: Some(row.usage),
            origin: Origin::Handler(handler),
        });
    }
    Ok(())
}

fn local<'f, 'a>(
    handler: L2IrHandler<'f, 'a>,
    id: HandlerLocalId,
) -> Result<HandlerLocalRef<'f, 'a>, Error> {
    let binding = handler
        .resolution()
        .and_then(|resolution| resolution.bindings().get(id.index() as usize))
        .filter(|row| row.id == id)
        .ok_or(Error::OriginalMembership)?;
    check_scope(handler, binding.scope)?;
    Ok(HandlerLocalRef { handler, binding })
}

fn check_scope(handler: L2IrHandler<'_, '_>, id: HandlerScopeId) -> Result<(), Error> {
    handler
        .resolution()
        .and_then(|resolution| resolution.scopes().get(id.index() as usize))
        .filter(|row| row.id == id)
        .map_or(Err(Error::OriginalMembership), |_| Ok(()))
}

fn for_head<'f, 'a>(
    head: L2IrForHead<'f, 'a>,
    visit: &mut impl FnMut(TemplateSiteRef<'f, 'a>),
) -> Result<(), Error> {
    let resolution = head.resolution().ok_or(Error::OriginalMembership)?;
    if !core::ptr::eq(
        resolution
            .input()
            .operand()
            .syntax()
            .source()
            .authored_root(),
        head.l2ir().artifact().source(),
    ) {
        return Err(Error::OriginalMembership);
    }
    let collection = resolution.collection();
    let span = resolution.collection_authored_span();
    checked_span(head.l2ir(), span)?;
    visit(TemplateSiteRef {
        symbol: TemplateSymbolRef::File(
            head.l2ir()
                .binding(collection.binding)
                .ok_or(Error::OriginalMembership)?,
        ),
        span,
        scope: TemplateSiteScope::File(head.enclosing_scope().ok_or(Error::OriginalMembership)?),
        usage: Some(collection.usage),
        origin: Origin::For(head),
    });
    let value = head.value().ok_or(Error::OriginalMembership)?;
    let key = head.key();
    if key.is_some() != resolution.key().is_some() {
        return Err(Error::OriginalMembership);
    }
    for binding in [Some(value), key].into_iter().flatten() {
        let row = binding
            .template_declaration()
            .ok_or(Error::OriginalMembership)?;
        let declaration = row.declaration();
        let original = declaration.original().ok_or(Error::OriginalMembership)?;
        if declaration.origin() != head.id()
            || Some(declaration.scope()) != head.scope()
            || !core::ptr::eq(original.resolution(), resolution)
        {
            return Err(Error::OriginalMembership);
        }
        let span = original.authored_span();
        checked_span(head.l2ir(), span)?;
        visit(TemplateSiteRef {
            symbol: TemplateSymbolRef::File(binding),
            span,
            scope: TemplateSiteScope::File(declaration.scope()),
            usage: None,
            origin: Origin::For(head),
        });
    }
    Ok(())
}

fn checked_span(l2ir: &L2Ir<'_>, span: Span) -> Result<(), Error> {
    if span.start >= span.end
        || l2ir
            .artifact()
            .source()
            .get(span.start as usize..span.end as usize)
            .is_none()
    {
        return Err(Error::Projection);
    }
    Ok(())
}
