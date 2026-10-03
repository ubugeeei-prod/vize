//! Static-name property decisions on the existing DOM owner frame.

use super::{DomBinding, DomBuilder, DomExpressionFacts, DomUnsupported, PropertyRole};
use crate::decision::dom::vue::policy::FileReads;
use vize_l0::id::NodeId;
use vize_l2::{
    op::{BindingOp, DynamicName, Op},
    walk::NodeRef,
};

impl<'owner, 'arena, F: DomExpressionFacts, R: FileReads<'owner, 'arena>>
    DomBuilder<'_, 'owner, 'arena, F, R>
{
    pub(in crate::decision) fn binding(&mut self, id: NodeId, binding: &'owner BindingOp<'arena>) {
        let span = NodeRef::Binding(binding).span();
        if let BindingOp::On(on) = binding {
            // Whole-handler custody is retained even while runtime semantics
            // remain refused by the existing bounded target policy below.
            self.record_handler(id, on);
        }
        let BindingOp::Bind(bind) = binding else {
            self.reject(id, span, DomUnsupported::Binding);
            return;
        };
        if !bind.modifiers.is_empty() {
            self.reject(id, span, DomUnsupported::BindingModifiers);
            return;
        }
        let Some(DynamicName::Static(name)) = bind.name else {
            self.reject(id, span, DomUnsupported::BindingName);
            return;
        };
        let role = match name {
            "class" => PropertyRole::Class,
            "style" => PropertyRole::Style,
            name if name.is_empty()
                || matches!(name, "key" | "ref" | "is")
                || name.starts_with("on") =>
            {
                self.reject(id, span, DomUnsupported::BindingName);
                return;
            }
            _ => PropertyRole::Property,
        };
        let Some((_, frame)) = self.frames.last() else {
            self.reject(id, span, DomUnsupported::Binding);
            return;
        };
        let Op::Element(element) = frame.node.op else {
            self.reject(id, span, DomUnsupported::Binding);
            return;
        };
        if frame.binding_names.contains(&name)
            || element
                .attributes
                .iter()
                .any(|attribute| attribute.name == name)
        {
            self.reject(id, span, DomUnsupported::DuplicateProperty);
            return;
        }
        let Some(expression) = bind.value else {
            self.reject(id, span, DomUnsupported::MissingValue);
            return;
        };
        let Some(value) = self.value(id, expression) else {
            return;
        };
        let Some((_, frame)) = self.frames.last_mut() else {
            self.reject(id, span, DomUnsupported::Binding);
            return;
        };
        frame.binding_names.push(name);
        frame.normalize_class |= role == PropertyRole::Class;
        frame.normalize_style |= role == PropertyRole::Style && value.is_dynamic();
        if value.is_dynamic() {
            match role {
                PropertyRole::Property => {
                    frame.node.changes.properties = true;
                    frame.node.dynamic_property_bindings.push(id);
                }
                PropertyRole::Class => frame.node.changes.class = true,
                PropertyRole::Style => frame.node.changes.style = true,
            }
        }
        self.facts.bindings.insert(
            id,
            DomBinding {
                binding,
                role,
                value,
            },
        );
    }
}
