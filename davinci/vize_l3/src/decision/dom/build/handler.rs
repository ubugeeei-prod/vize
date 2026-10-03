//! Join the complete lower owner during the sole canonical binding walk.

use super::{DomBuilder, DomExpressionFacts};
use crate::decision::dom::{DomUnsupported, handler::DomFileHandler, vue::policy::FileReads};
use vize_l0::id::NodeId;
use vize_l2::op::OnOp;

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
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
