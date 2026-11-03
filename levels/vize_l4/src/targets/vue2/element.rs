use super::{
    Emitter, Vue2Error, Vue2ErrorKind,
    quote::{property, quoted},
};
use crate::write::{LinkSink, Writer};
use vize_l0::id::NodeId;
use vize_l2::op::ElementOp;
use vize_l3::decision::dom::DomChildren;

impl<L: LinkSink> Emitter<'_, '_, '_, L> {
    pub(super) fn element(
        &mut self,
        node: NodeId,
        element: &ElementOp<'_>,
        children: &DomChildren,
    ) -> Result<(), Vue2Error> {
        if !element.bindings.is_empty() {
            return Err(self.error(node, Vue2ErrorKind::UnsupportedBindings));
        }
        self.writer.push(self.receiver);
        self.writer.push("._c(");
        quoted(&mut self.writer, element.tag, element.span);
        if element.attributes.is_empty() && matches!(children, DomChildren::Empty) {
            self.writer.push(")");
            return Ok(());
        }
        self.writer.push(", ");
        if element.attributes.is_empty() {
            self.writer.push("void 0");
        } else {
            let mut attributes = Writer::<L>::default();
            let mut class = None;
            for attr in &element.attributes {
                if attr.name == "class" {
                    class = Some(attr);
                    continue;
                }
                if !attributes.is_empty() {
                    attributes.push(", ");
                }
                property(&mut attributes, attr.name, attr.span);
                attributes.push(": ");
                quoted(&mut attributes, attr.value.unwrap_or(""), attr.span);
            }
            self.writer.push("{");
            // Vue 2 keeps the element's data object for an authored class,
            // but its class transform omits a falsy staticClass field.
            if let Some(class) =
                class.filter(|class| class.value.is_some_and(|value| !value.is_empty()))
            {
                self.writer.push("staticClass: ");
                quoted(&mut self.writer, class.value.unwrap_or(""), class.span);
                if !attributes.is_empty() {
                    self.writer.push(", ");
                }
            }
            if !attributes.is_empty() {
                self.writer.push("attrs: {");
                self.writer.append(attributes);
                self.writer.push("}");
            }
            self.writer.push("}");
        }
        if !matches!(children, DomChildren::Empty) {
            self.writer.push(", ");
            self.children(children)?;
        }
        self.writer.push(")");
        Ok(())
    }
}
