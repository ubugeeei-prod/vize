//! Join the complete lower owner during the sole canonical binding walk.

use super::{DomBinding, DomBuilder, DomExpressionFacts, PropertyRole, ValueKind};
use crate::decision::dom::{DomUnsupported, handler::DomFileHandler, vue::policy::FileReads};
use vize_l0::id::NodeId;
use vize_l2::op::{BindingOp, DynamicName, OnOp, Op};

mod access;

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(super) fn handler_binding(
        &mut self,
        node: NodeId,
        binding: &'owner BindingOp<'arena>,
        on: &'owner OnOp<'arena>,
    ) {
        self.record_handler(node, on);
        let Some(row) = self.facts.file_handlers.get(node) else {
            self.reject(node, on.span, DomUnsupported::Binding);
            return;
        };
        let resolution = row.resolution();
        let source = resolution.input().operand().syntax().source();
        if let Some(returned) = resolution.syntax().first_return() {
            let span = source
                .authored_covering_span(returned)
                .unwrap_or(source.span());
            self.reject(node, span, DomUnsupported::HandlerSyntax);
            return;
        }
        if !on.modifiers.is_empty() {
            self.reject(node, on.span, DomUnsupported::BindingModifiers);
            return;
        }
        if !matches!(on.name, Some(DynamicName::Static("click"))) {
            self.reject(node, on.span, DomUnsupported::BindingName);
            return;
        }
        // This real syntax family is unambiguously an inline statement body.
        // Single member/identifier handler-reference syntax is a separate policy.
        if !resolution.syntax().leading_declaration()
            || !source.text().contains(';')
            || resolution
                .input()
                .operand()
                .syntax()
                .comments()
                .any(|comment| comment.text().map_or(true, |text| text.starts_with("//")))
        {
            self.reject(node, source.span(), DomUnsupported::HandlerSyntax);
            return;
        }
        if resolution.references().is_empty() {
            self.reject(node, source.span(), DomUnsupported::HandlerAccess);
            return;
        }
        // Event parameters and genuine active-block locals retain original
        // spelling. Outer/setup and Vue-prefixed root/sibling reads still refuse.
        for (index, reference) in resolution.references().iter().enumerate() {
            if !access::original(resolution, index, reference) {
                let span = source
                    .authored_covering_span(reference.span)
                    .unwrap_or(source.span());
                self.reject(node, span, DomUnsupported::HandlerAccess);
                return;
            }
        }
        let Some((_, frame)) = self.frames.last_mut() else {
            self.reject(node, on.span, DomUnsupported::Binding);
            return;
        };
        let Op::Element(element) = frame.node.op else {
            self.reject(node, on.span, DomUnsupported::Binding);
            return;
        };
        if let Some(attribute) = element
            .attributes
            .iter()
            .find(|attribute| attribute.span.start > on.span.start)
        {
            self.reject(node, attribute.span, DomUnsupported::HandlerSyntax);
            return;
        }
        if frame.binding_names.contains(&"onClick")
            || element
                .attributes
                .iter()
                .any(|attribute| attribute.name.eq_ignore_ascii_case("onClick"))
        {
            self.reject(node, on.span, DomUnsupported::DuplicateProperty);
            return;
        }
        frame.node.changes.properties = true;
        frame.node.dynamic_property_bindings.push(node);
        self.facts.bindings.insert(
            node,
            DomBinding {
                binding,
                role: PropertyRole::Event,
                value: ValueKind::FileDependent,
            },
        );
    }
    pub(super) fn record_handler(&mut self, node: NodeId, on: &'owner OnOp<'arena>) {
        // Bare-artifact policies cannot authenticate original File/body custody.
        let Some(file) = self.file else { return };
        let Some(handler) = file
            .handler_for(on)
            .filter(|handler| handler.id().node() == node)
        else {
            self.reject(node, on.span, DomUnsupported::FileHandler);
            return;
        };
        if !handler.scope().is_some_and(|scope| {
            file.scopes()
                .get(scope.index() as usize)
                .is_some_and(|row| row.id == scope)
        }) {
            self.reject(node, on.span, DomUnsupported::FileScope);
            return;
        }
        let Some(resolution) = handler.resolution() else {
            self.reject(node, on.span, DomUnsupported::FileHandler);
            return;
        };
        let source = resolution.input().operand().syntax().source();
        if !core::ptr::eq(source.authored_root(), file.artifact().source()) {
            self.reject(node, on.span, DomUnsupported::FileHandler);
            return;
        }
        self.facts.file_handlers.insert(
            node,
            DomFileHandler {
                handler,
                on,
                resolution,
            },
        );
    }
}
