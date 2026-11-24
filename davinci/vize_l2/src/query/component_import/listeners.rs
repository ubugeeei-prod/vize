//! Original static event names, handler text and modifiers on one resolved component.
use super::ImportedComponentSite;
use crate::op::{BindingOp, DynamicName, OnHandlerRef};
use vize_l0::{SourceRoot, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentListenerIssue {
    OriginalSource,
    UnsupportedBinding(Span),
}

#[derive(Clone, Copy)]
pub struct ImportedComponentListener<'f, 'a> {
    pub name: &'a str,
    pub name_span: Span,
    pub handler: Option<Span>,
    /// Declaration of the original method reference in this File.
    pub handler_binding: Option<crate::query::BindingRef<'f, 'a>>,
    pub modifiers: &'f [&'a str],
    pub span: Span,
}

impl<'f, 'a> ImportedComponentSite<'f, 'a> {
    /// Visit all static listener rows on this owner in authored order. A
    /// refusal invalidates the entire visit, including any preceding rows.
    pub fn visit_static_listeners(
        &self,
        mut visit: impl FnMut(ImportedComponentListener<'f, 'a>),
    ) -> Result<(), ComponentListenerIssue> {
        let file = self.binding.file();
        let root = SourceRoot::new(file.artifact().source())
            .map_err(|_| ComponentListenerIssue::OriginalSource)?
            .whole_block();
        let mut previous = self.name_span().end;
        // Attached bindings occupy consecutive File nodes immediately after their owner.
        let mut node = self.node();
        for binding in &self.component.bindings {
            node = node.next().ok_or(ComponentListenerIssue::OriginalSource)?;
            let on = match binding {
                BindingOp::Bind(_) => continue,
                BindingOp::On(on) => on,
                other => {
                    return Err(ComponentListenerIssue::UnsupportedBinding(
                        crate::walk::NodeRef::Binding(other).span(),
                    ));
                }
            };
            let Some(DynamicName::Static(name)) = on.name else {
                return Err(ComponentListenerIssue::UnsupportedBinding(on.span));
            };
            let name_span = root
                .span_of(name)
                .ok_or(ComponentListenerIssue::OriginalSource)?;
            if on.span.start < previous
                || on.span.end > self.element_span().end
                || name_span.start < on.span.start
                || name_span.end > on.span.end
            {
                return Err(ComponentListenerIssue::OriginalSource);
            }
            let (handler, handler_binding) = match on.handler {
                None => (None, None),
                Some(OnHandlerRef::Js(js)) => {
                    let original = ComponentListenerIssue::OriginalSource;
                    let table = file
                        .expression(node)
                        .and_then(|row| row.table())
                        .ok_or(original)?;
                    let [occurrence] = table.occurrences() else {
                        return Err(ComponentListenerIssue::UnsupportedBinding(on.span));
                    };
                    if !core::ptr::eq(table.expression().ast, js.ast)
                        || js.authored_span(occurrence.span) != Some(js.span)
                        || !root.contains_block_span(js.span)
                        || js.span.start < name_span.end
                        || js.span.end > on.span.end
                        || js.span.slice(root.source()) != occurrence.name
                    {
                        return Err(original);
                    }
                    (
                        Some(js.span),
                        Some(file.binding(occurrence.binding).ok_or(original)?),
                    )
                }
                _ => return Err(ComponentListenerIssue::UnsupportedBinding(on.span)),
            };
            visit(ImportedComponentListener {
                name,
                name_span,
                handler,
                handler_binding,
                modifiers: &on.modifiers,
                span: on.span,
            });
            previous = on.span.end;
        }
        Ok(())
    }
}
