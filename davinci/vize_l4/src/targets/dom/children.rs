//! Encode L3's already grouped child sequences.

use vize_l2::op::Op;
use vize_l3::decision::dom::{DomChild, DomText};

use super::write::{helper, quoted, spell};
use super::{DomError, DomErrorKind, Emitter, ExpressionWriter, LinkSink};

impl<E: ExpressionWriter, L: LinkSink> Emitter<'_, '_, '_, E, L> {
    pub(super) fn root_fragment(&mut self, single_non_comment: bool) -> Result<(), DomError> {
        self.writer.push("(");
        spell(&mut self.writer, self.vocabulary, self.helpers.open_block);
        self.writer.push("(), ");
        spell(
            &mut self.writer,
            self.vocabulary,
            self.helpers.element_block,
        );
        self.writer.push("(");
        spell(&mut self.writer, self.vocabulary, self.helpers.fragment);
        self.writer.push(", null, ");
        self.array(&self.facts.root().children)?;
        self.writer.use_helper(self.helpers.fragment);
        self.writer.use_helper(self.helpers.open_block);
        self.writer.use_helper(self.helpers.element_block);
        self.writer.push(if single_non_comment {
            ", 2112 /* STABLE_FRAGMENT, DEV_ROOT_FRAGMENT */))"
        } else {
            ", 64 /* STABLE_FRAGMENT */))"
        });
        Ok(())
    }

    pub(super) fn child(&mut self, child: &DomChild, vnode: bool) -> Result<(), DomError> {
        match child {
            DomChild::Node(node) => self.node(*node),
            DomChild::Text(text) if vnode => self.text_vnode(text),
            DomChild::Text(text) => self.text(text),
        }
    }

    pub(super) fn array(&mut self, children: &[DomChild]) -> Result<(), DomError> {
        self.writer.push("[");
        self.writer.indent();
        let mut text_vnode = false;
        for (index, child) in children.iter().enumerate() {
            if index != 0 {
                self.writer.push(",");
            }
            self.writer.newline();
            self.child(child, true)?;
            text_vnode |= matches!(child, DomChild::Text(_));
        }
        self.writer.deindent();
        self.writer.newline();
        self.writer.push("]");
        // Vue's text transformation closes after all siblings, including nodes
        // written after the first text group. This adds no second child walk.
        if text_vnode {
            self.writer.use_helper(self.helpers.text);
        }
        Ok(())
    }

    fn text_vnode(&mut self, text: &DomText) -> Result<(), DomError> {
        spell(&mut self.writer, self.vocabulary, self.helpers.text);
        let single_space = match text.nodes.as_slice() {
            [node] if !text.dynamic => self
                .facts
                .node(*node)
                .is_some_and(|fact| matches!(fact.op(), Op::Text(text) if text.content == " ")),
            _ => false,
        };
        if single_space {
            self.writer.push("()");
            return Ok(());
        }
        self.writer.push("(");
        self.text(text)?;
        if text.dynamic {
            self.writer.push(", 1 /* TEXT */");
        }
        self.writer.push(")");
        Ok(())
    }

    pub(super) fn text(&mut self, group: &DomText) -> Result<(), DomError> {
        for (index, &node) in group.nodes.iter().enumerate() {
            if index != 0 {
                self.writer.push(" + ");
            }
            let fact = self
                .facts
                .node(node)
                .ok_or_else(|| self.error(node, DomErrorKind::MissingNode))?;
            match fact.op() {
                Op::Text(text) => quoted(&mut self.writer, text.content, text.span),
                Op::Interpolation(interpolation) => {
                    helper(&mut self.writer, self.vocabulary, self.helpers.display);
                    self.writer.push("(");
                    self.expression(node, interpolation.expression)?;
                    self.writer.push(")");
                }
                _ => return Err(self.error(node, DomErrorKind::InvalidGrouping)),
            }
        }
        Ok(())
    }
}
