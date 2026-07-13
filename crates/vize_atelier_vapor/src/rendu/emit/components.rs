use super::super::{VaporBlockId, VaporName, VaporOperation, VaporProperty};
use super::Emitter;
use super::directive::emit_component_directives;
use super::property::{expression, name, object_key, props_object, quote_js, use_helper};

impl Emitter<'_> {
    pub(super) fn emit_component(
        &mut self,
        component: &VaporName,
        properties: &[VaporProperty],
        body: VaporBlockId,
        indent: usize,
        out: &mut String,
    ) -> String {
        use_helper(&mut self.helpers, "createComponentWithFallback");
        if matches!(component, VaporName::Static(_)) {
            use_helper(&mut self.helpers, "resolveComponent");
        }
        if properties.iter().any(|property| {
            matches!(
                property,
                VaporProperty::Directive(directive)
                    if directive.name.as_ref() == "on" && !directive.modifiers.is_empty()
            )
        }) {
            use_helper(&mut self.helpers, "withModifiers");
        }
        let variable = self.node();
        let component = match component {
            VaporName::Static(component) => cstr!("_resolveComponent({})", quote_js(component)),
            VaporName::Dynamic(component) => expression(self.plan, *component).to_compact_string(),
        };
        let props = props_object(self.plan, properties, true);
        let slots = self.component_slots(body, indent);
        self.line(
            out,
            indent,
            &cstr!(
                "const {variable} = _createComponentWithFallback({component}, {props}, {slots}, true)"
            ),
        );
        emit_component_directives(
            self.plan,
            properties,
            &variable,
            indent,
            out,
            &mut self.helpers,
        );
        variable
    }

    pub(super) fn emit_slot(
        &mut self,
        slot: &VaporName,
        properties: &[VaporProperty],
        fallback: VaporBlockId,
        indent: usize,
        out: &mut String,
    ) -> String {
        use_helper(&mut self.helpers, "renderSlot");
        let variable = self.node();
        let callback = self.callback(fallback, "", indent);
        self.line(
            out,
            indent,
            &cstr!(
                "const {variable} = _renderSlot($slots, {}, {}, {callback})",
                name(self.plan, slot),
                props_object(self.plan, properties, false)
            ),
        );
        variable
    }

    fn component_slots(&mut self, body: VaporBlockId, indent: usize) -> String {
        let operations = &self
            .plan
            .block(body)
            .expect("validated Vapor block")
            .operations;
        let explicit = operations
            .iter()
            .filter_map(|operation| match operation {
                VaporOperation::SlotContent {
                    name: slot,
                    bindings,
                    body,
                    ..
                } => Some((slot, bindings, *body)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if explicit.is_empty() {
            return cstr!("{{ default: {} }}", self.callback(body, "", indent));
        }
        let mut slots = Vec::new();
        for (slot, bindings, body) in explicit {
            let params = bindings
                .iter()
                .map(|binding| binding.pattern.as_ref())
                .collect::<Vec<_>>()
                .join(", ");
            slots.push(cstr!(
                "{}: {}",
                object_key(self.plan, slot),
                self.callback(body, &params, indent)
            ));
        }
        cstr!("{{ {} }}", slots.join(", "))
    }
}
use vize_carton::{String, ToCompactString, cstr};
