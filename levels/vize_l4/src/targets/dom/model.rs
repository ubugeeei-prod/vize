//! Runtime text-model realization from a distinct authenticated read/write row.

use vize_l0::id::NodeId;
use vize_l2::file::NativeFileModelKind;
use vize_l2::op::{BindingOp, ElementOp, ModelOp};
use vize_l3::decision::dom::{DomChild, DomFileModel, PropertyRole};

use super::write::spell;
use super::{DomError, DomErrorKind, Emitter, ExpressionWriter, LinkSink};
use crate::write::Writer;

impl<E: ExpressionWriter, L: LinkSink> Emitter<'_, '_, '_, E, L> {
    /// Both spans already belong to the checked original Model/Element row.
    /// They choose property order only and confer no source admission.
    pub(super) fn model_precedes_type(
        &self,
        node: NodeId,
        element: &ElementOp<'_>,
    ) -> Result<bool, DomError> {
        let fail = || self.error(node, DomErrorKind::FileOwnerMismatch);
        let row = self.facts.file_model(node).ok_or_else(fail)?;
        match row.original().kind() {
            Some(NativeFileModelKind::Text) => Ok(false),
            Some(NativeFileModelKind::Checkbox) => {
                let [attribute] = element.attributes.as_slice() else {
                    return Err(fail());
                };
                Ok(row.model().span.start < attribute.span.start)
            }
            None => Err(fail()),
        }
    }

    pub(super) fn model_start(&mut self, node: NodeId) -> Result<(), DomError> {
        let helper = self
            .vocabulary
            .helper("withDirectives")
            .ok_or_else(|| self.error(node, DomErrorKind::MissingRuntimeHelper))?;
        spell(&mut self.writer, self.vocabulary, helper);
        self.writer.push("(");
        Ok(())
    }
    pub(super) fn element_model(
        &self,
        node: NodeId,
        element: &ElementOp<'_>,
        branch_key: Option<u32>,
    ) -> Result<Option<NodeId>, DomError> {
        let [BindingOp::Model(model)] = element.bindings.as_slice() else {
            return Ok(None);
        };
        let fail = || self.error(node, DomErrorKind::FileOwnerMismatch);
        let [DomChild::Node(root)] = self.facts.root().children.as_slice() else {
            return Err(fail());
        };
        let ids = &self
            .tables
            .nodes
            .get(node)
            .ok_or_else(fail)?
            .dynamic_bindings;
        let [model_node] = ids.as_slice() else {
            return Err(fail());
        };
        let row = self.facts.file_model(*model_node).ok_or_else(fail)?;
        if *root != node || branch_key.is_some() || element.tag != "input"
            || !element.children.ops.is_empty()
            || !row.accepts_model(model) || !row.accepts_element(node, element)
            || !self.facts.binding(*model_node).is_some_and(|fact| {
                fact.role == PropertyRole::Model
                    && matches!(fact.binding(), BindingOp::Model(actual) if core::ptr::eq(&**actual, &**model))
            })
        {
            return Err(fail());
        }
        Ok(Some(*model_node))
    }

    pub(super) fn model_update(
        &mut self,
        node: NodeId,
        model: &ModelOp<'_>,
    ) -> Result<(), DomError> {
        let row = self
            .facts
            .file_model(node)
            .filter(|row| row.accepts_model(model))
            .ok_or_else(|| self.error(node, DomErrorKind::FileOwnerMismatch))?;
        // These tokens implement the runtime callback. No assignment AST or
        // synthetic authored span is introduced into File or resolution facts.
        self.writer.push("$event => ((");
        access(&mut self.writer, &row);
        self.writer.push(") = $event)");
        Ok(())
    }

    pub(super) fn model_directive(&mut self, node: NodeId) -> Result<(), DomError> {
        let row = self
            .facts
            .file_model(node)
            .ok_or_else(|| self.error(node, DomErrorKind::MissingBinding))?;
        let helper_name = match row.original().kind() {
            Some(NativeFileModelKind::Text) => "vModelText",
            Some(NativeFileModelKind::Checkbox) => "vModelCheckbox",
            None => return Err(self.error(node, DomErrorKind::FileOwnerMismatch)),
        };
        let model_text = self
            .vocabulary
            .helper(helper_name)
            .ok_or_else(|| self.error(node, DomErrorKind::MissingRuntimeHelper))?;
        let with_directives = self
            .vocabulary
            .helper("withDirectives")
            .ok_or_else(|| self.error(node, DomErrorKind::MissingRuntimeHelper))?;
        self.writer.push(", [");
        self.writer.indent();
        self.writer.newline();
        self.writer.push("[");
        spell(&mut self.writer, self.vocabulary, model_text);
        self.writer.push(", ");
        access(&mut self.writer, &row);
        let operand = row.original().input().operand();
        let modifier = operand
            .trim_span()
            .map(|span| ("trim", span))
            .or_else(|| operand.lazy_span().map(|span| ("lazy", span)))
            .or_else(|| operand.number_span().map(|span| ("number", span)));
        if let Some((name, span)) = modifier {
            self.writer.push(", void 0, { ");
            self.writer.push_named(name, span, name);
            self.writer.push(": true }");
        }
        self.writer.push("]");
        self.writer.deindent();
        self.writer.newline();
        self.writer.push("])");
        self.writer.use_helper(model_text);
        self.writer.use_helper(with_directives);
        Ok(())
    }
}

fn access<L: LinkSink>(writer: &mut Writer<L>, row: &DomFileModel<'_, '_>) {
    // The sealed row already joins the actual direct mutable setup declaration,
    // sole ReadWrite occurrence and complete original Model to this File.
    let occurrence = row.occurrence();
    writer.push_named_parts(
        ["$setup.", occurrence.name],
        row.authored_span(),
        occurrence.name,
    );
}
