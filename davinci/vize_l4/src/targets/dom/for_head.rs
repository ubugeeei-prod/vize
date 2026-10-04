//! Encode only the original root For carrier sealed by the DOM walk.

use super::{DomError, DomErrorKind, Emitter, ExpressionWriter, LinkSink, write::spell};
use vize_l0::id::NodeId;
use vize_l2::op::OriginalForOp;
use vize_l3::decision::dom::{DomChild, DomChildren, DomNode};

impl<E: ExpressionWriter, L: LinkSink> Emitter<'_, '_, '_, E, L> {
    pub(super) fn original_for(
        &mut self,
        node: NodeId,
        original: &OriginalForOp<'_>,
        fact: &DomNode<'_, '_>,
    ) -> Result<(), DomError> {
        let row = self
            .facts
            .file_for_head(node)
            .filter(|row| row.accepts_original(original))
            .ok_or_else(|| self.error(node, DomErrorKind::FileOwnerMismatch))?;
        if !fact.block_eligible {
            return Err(self.error(node, DomErrorKind::RuntimeAccessUnavailable));
        }
        let DomChildren::Array(children) = &fact.children else {
            return Err(self.error(node, DomErrorKind::InvalidGrouping));
        };
        let [DomChild::Node(body)] = children.as_slice() else {
            return Err(self.error(node, DomErrorKind::InvalidGrouping));
        };
        let alias = row.resolution().value_declaration();
        let name = alias.fact().name();
        // No generated local is introduced inside this callback. All helper
        // aliases actually used inside it must retain their original meaning.
        if name.strip_prefix('_').is_some_and(|name| {
            [
                self.helpers.open_block,
                self.helpers.element_block,
                self.helpers.display,
            ]
            .into_iter()
            .any(|helper| self.vocabulary.name(helper) == Some(name))
        }) {
            return Err(DomError {
                node: Some(node),
                span: alias.authored_span(),
                kind: DomErrorKind::GeneratedForBindingCollision,
            });
        }
        let render_list = self
            .vocabulary
            .helper("renderList")
            .ok_or_else(|| self.error(node, DomErrorKind::MissingRuntimeHelper))?;
        self.writer.anchor(original.span.start);
        self.writer.push("(");
        spell(&mut self.writer, self.vocabulary, self.helpers.open_block);
        self.writer.push("(true), ");
        spell(
            &mut self.writer,
            self.vocabulary,
            self.helpers.element_block,
        );
        self.writer.push("(");
        spell(&mut self.writer, self.vocabulary, self.helpers.fragment);
        self.writer.push(", null, ");
        spell(&mut self.writer, self.vocabulary, render_list);
        self.writer.push("(");
        self.expressions
            .write_for_collection(&mut self.writer, node, original)?;
        self.writer.push(", (");
        self.writer.push_named(name, alias.authored_span(), name);
        self.writer.push(") => {");
        self.writer.indent();
        self.writer.newline();
        self.writer.push("return ");
        self.node(*body)?;
        self.writer.deindent();
        self.writer.newline();
        self.writer.push("}), 256 /* UNKEYED_FRAGMENT */))");
        Ok(())
    }
}
