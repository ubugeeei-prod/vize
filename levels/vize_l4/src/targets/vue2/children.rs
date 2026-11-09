use super::{Emitter, Vue2Error, Vue2ErrorKind, quote::quoted};
use crate::write::LinkSink;
use vize_l2::op::Op;
use vize_l3::decision::dom::{DomChild, DomChildren, DomText};

impl<L: LinkSink> Emitter<'_, '_, '_, L> {
    pub(super) fn children(&mut self, children: &DomChildren) -> Result<(), Vue2Error> {
        self.writer.push("[");
        match children {
            DomChildren::Empty => {}
            DomChildren::Text(text) => self.text(text)?,
            DomChildren::Array(children) => {
                for (index, child) in children.iter().enumerate() {
                    if index != 0 {
                        self.writer.push(", ");
                    }
                    match child {
                        DomChild::Node(node) => self.node(*node)?,
                        DomChild::Text(text) => self.text(text)?,
                    }
                }
            }
        }
        self.writer.push("]");
        Ok(())
    }
    fn text(&mut self, group: &DomText) -> Result<(), Vue2Error> {
        self.writer.push(self.receiver);
        self.writer.push("._v(");
        for (index, &node) in group.nodes.iter().enumerate() {
            if index != 0 {
                self.writer.push(" + ");
            }
            let fact = self
                .facts
                .node(node)
                .ok_or_else(|| self.error(node, Vue2ErrorKind::MissingNode))?;
            match fact.op() {
                Op::Text(text) => quoted(&mut self.writer, text.content, text.span),
                Op::Interpolation(interpolation) => {
                    self.writer.push(self.receiver);
                    self.writer.push("._s(");
                    self.expression(node, interpolation.expression)?;
                    self.writer.push(")");
                }
                _ => return Err(self.error(node, Vue2ErrorKind::UnsupportedOperation)),
            }
        }
        self.writer.push(")");
        Ok(())
    }
}
