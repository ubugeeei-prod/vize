use vize_rendu::{
    RenduAttributeValue, RenduComponentKind, RenduExpressionId, RenduName, RenduNode, RenduNodeId,
    RenduProperty,
};

use super::SsrEmitter;

#[derive(Clone, Copy)]
enum TransitionGroupTag<'a> {
    Static(&'a str),
    Dynamic(RenduExpressionId),
}

impl SsrEmitter<'_> {
    pub(super) fn emit_component(
        &mut self,
        kind: RenduComponentKind,
        name: &RenduName,
        properties: &[RenduProperty],
        children: &[RenduNodeId],
        fallthrough: bool,
    ) {
        match kind {
            RenduComponentKind::Suspense => {
                self.emit_suspense(children);
                return;
            }
            RenduComponentKind::Teleport => {
                self.emit_teleport(properties, children);
                return;
            }
            RenduComponentKind::KeepAlive | RenduComponentKind::Transition => {
                self.emit_default_slot_children(children, fallthrough);
                return;
            }
            RenduComponentKind::TransitionGroup => {
                self.emit_transition_group(properties, children, fallthrough);
                return;
            }
            RenduComponentKind::Ordinary | RenduComponentKind::Dynamic => {}
        }
        self.indent();
        self.output.code.push_str("_push(_ssrRenderComponent(");
        self.emit_component_name(kind, name, properties);
        self.output.code.push_str(", ");
        self.emit_component_properties_with_fallthrough(kind, properties, fallthrough);
        self.output.code.push_str(", ");
        self.emit_slots(children);
        self.output.code.push_str(", _parent");
        if self.slot_scope_depth > 0 {
            self.output.code.push_str(", _scopeId");
        }
        self.output.code.push_str("))\n");
    }

    fn emit_teleport(&mut self, properties: &[RenduProperty], children: &[RenduNodeId]) {
        self.indent();
        self.output
            .code
            .push_str("_ssrRenderTeleport(_push, (_push) => {\n");
        self.indent += 1;
        self.emit_default_slot_children(children, false);
        self.indent -= 1;
        self.indent();
        self.output.code.push_str("}, ");
        self.emit_named_property(properties, "to", "undefined");
        self.output.code.push_str(", ");
        self.emit_named_property(properties, "disabled", "false");
        self.output.code.push_str(", _parent)\n");
    }

    fn emit_default_slot_children(&mut self, children: &[RenduNodeId], fallthrough: bool) {
        let mut default_children = Vec::new();
        for &child in children {
            match self.root.node(child).expect("validated component child") {
                RenduNode::SlotContent {
                    name: RenduName::Static(name),
                    children,
                    ..
                } if name.as_ref() == "default" => {
                    default_children.extend(children.iter().copied());
                }
                RenduNode::SlotContent { .. } => {}
                _ => default_children.push(child),
            }
        }
        if fallthrough {
            self.emit_root_nodes(&default_children);
        } else {
            self.emit_nodes(&default_children);
        }
    }

    fn emit_transition_group(
        &mut self,
        properties: &[RenduProperty],
        children: &[RenduNodeId],
        fallthrough: bool,
    ) {
        let Some(tag) = self.transition_group_tag(properties) else {
            self.push_line_value("<!--[-->");
            self.emit_default_slot_children(children, false);
            self.push_line_value("<!--]-->");
            return;
        };
        let filtered = properties
            .iter()
            .filter(|property| !super::property::is_named_property(property, "tag"))
            .cloned()
            .collect::<Vec<_>>();
        match tag {
            TransitionGroupTag::Static(tag) => {
                self.push_line_value(&vize_carton::cstr!("<{tag}"));
            }
            TransitionGroupTag::Dynamic(tag) => {
                self.push_line_value("<");
                self.push_dynamic_tag(tag);
            }
        }
        if !filtered.is_empty() || fallthrough {
            self.indent();
            self.output.code.push_str("_push(_ssrRenderAttrs(");
            self.emit_component_properties_with_fallthrough(
                RenduComponentKind::Ordinary,
                &filtered,
                fallthrough,
            );
            self.output.code.push_str("))\n");
        }
        if let Some(scope_id) = self.root.component_scope_id() {
            self.push_line_value(&vize_carton::cstr!(" {scope_id}"));
        }
        self.push_line_value(">");
        self.emit_default_slot_children(children, false);
        match tag {
            TransitionGroupTag::Static(tag) => {
                self.push_line_value(&vize_carton::cstr!("</{tag}>"));
            }
            TransitionGroupTag::Dynamic(tag) => {
                self.push_line_value("</");
                self.push_dynamic_tag(tag);
                self.push_line_value(">");
            }
        }
    }

    fn transition_group_tag<'a>(
        &self,
        properties: &'a [RenduProperty],
    ) -> Option<TransitionGroupTag<'a>> {
        properties.iter().find_map(|property| match property {
            RenduProperty::Attribute(attribute)
                if matches!(&attribute.name, RenduName::Static(key) if key.as_ref() == "tag") =>
            {
                match attribute.value.as_ref() {
                    Some(RenduAttributeValue::Static(value)) => {
                        Some(TransitionGroupTag::Static(value))
                    }
                    Some(RenduAttributeValue::Expression(expression)) => {
                        Some(TransitionGroupTag::Dynamic(*expression))
                    }
                    _ => None,
                }
            }
            RenduProperty::Directive(directive)
                if directive.name.as_ref() == "bind"
                    && matches!(&directive.argument, Some(RenduName::Static(key)) if key.as_ref() == "tag") =>
            {
                directive.expression.map(TransitionGroupTag::Dynamic)
            }
            _ => None,
        })
    }

    fn push_dynamic_tag(&mut self, tag: RenduExpressionId) {
        self.indent();
        self.output.code.push_str("_push(String(");
        self.emit_expression(tag);
        self.output.code.push_str("))\n");
    }

    fn emit_named_property(&mut self, properties: &[RenduProperty], name: &str, fallback: &str) {
        for property in properties {
            match property {
                RenduProperty::Attribute(attribute) if matches!(&attribute.name, RenduName::Static(key) if key.as_ref() == name) =>
                {
                    match attribute.value.as_ref() {
                        None => self.output.code.push_str("true"),
                        Some(RenduAttributeValue::Static(value)) => {
                            super::syntax::quote_js(&mut self.output.code, value);
                        }
                        Some(RenduAttributeValue::Expression(expression)) => {
                            self.emit_expression(*expression);
                        }
                    }
                    return;
                }
                RenduProperty::Directive(directive)
                    if directive.name.as_ref() == "bind"
                        && matches!(&directive.argument, Some(RenduName::Static(key)) if key.as_ref() == name) =>
                {
                    if let Some(expression) = directive.expression {
                        self.emit_expression(expression);
                        return;
                    }
                }
                _ => {}
            }
        }
        self.output.code.push_str(fallback);
    }

    pub(super) fn emit_slot_outlet(
        &mut self,
        name: &RenduName,
        properties: &[RenduProperty],
        fallback: &[RenduNodeId],
    ) {
        self.indent();
        self.output.code.push_str("_ssrRenderSlot(_ctx.$slots, ");
        self.emit_name_value(name);
        self.output.code.push_str(", ");
        self.emit_properties(properties);
        self.output.code.push_str(", () => {\n");
        self.indent += 1;
        self.emit_nodes(fallback);
        self.indent -= 1;
        self.indent();
        self.output.code.push_str("}, _push, _parent");
        if self.slot_scope_depth > 0 {
            self.output.code.push_str(", _scopeId");
        }
        self.output.code.push_str(")\n");
    }
}
