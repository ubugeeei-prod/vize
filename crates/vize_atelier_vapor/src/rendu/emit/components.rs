use super::super::{
    VaporBlockId, VaporComponentSlots, VaporConditionalSlotBranch, VaporDynamicSlot, VaporName,
    VaporOperation, VaporProperty, VaporSlot,
};
use super::Emitter;
use super::directive::emit_component_directives;
use super::property::{
    expression, is_named_property, name, object_key, props_object, quote_js, use_helper,
};
use vize_carton::{String, ToCompactString, cstr};
use vize_rendu::RenduComponentKind;

impl Emitter<'_> {
    pub(super) fn emit_component(
        &mut self,
        kind: RenduComponentKind,
        component: &VaporName,
        properties: &[VaporProperty],
        slots: &VaporComponentSlots,
        indent: usize,
        out: &mut String,
    ) -> String {
        let filtered_properties = properties
            .iter()
            .filter(|property| {
                kind != RenduComponentKind::Dynamic || !is_named_property(property, "is")
            })
            .cloned()
            .collect::<Vec<_>>();
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
        let (create, component) = match kind {
            RenduComponentKind::Ordinary => {
                use_helper(&mut self.helpers, "createComponentWithFallback");
                let component = match component {
                    VaporName::Static(component) => {
                        use_helper(&mut self.helpers, "resolveComponent");
                        cstr!("_resolveComponent({})", quote_js(component))
                    }
                    VaporName::Dynamic(component) => {
                        expression(self.plan, *component).to_compact_string()
                    }
                };
                ("createComponentWithFallback", component)
            }
            RenduComponentKind::Dynamic => {
                use_helper(&mut self.helpers, "createDynamicComponent");
                let value = dynamic_component_value(self.plan, properties);
                ("createDynamicComponent", cstr!("() => ({value})"))
            }
            built_in => {
                use_helper(&mut self.helpers, "createComponent");
                let helper = match built_in {
                    RenduComponentKind::Suspense => "Suspense",
                    RenduComponentKind::Teleport => "VaporTeleport",
                    RenduComponentKind::KeepAlive => "VaporKeepAlive",
                    RenduComponentKind::Transition => "VaporTransition",
                    RenduComponentKind::TransitionGroup => "VaporTransitionGroup",
                    RenduComponentKind::Ordinary | RenduComponentKind::Dynamic => unreachable!(),
                };
                use_helper(&mut self.helpers, helper);
                ("createComponent", cstr!("_{helper}"))
            }
        };
        let props = props_object(self.plan, &filtered_properties, true);
        let slots = self.component_slots(slots, indent);
        self.line(
            out,
            indent,
            &cstr!("const {variable} = _{create}({component}, {props}, {slots}, true)"),
        );
        emit_component_directives(
            self.plan,
            &filtered_properties,
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

    fn component_slots(&mut self, slots: &VaporComponentSlots, indent: usize) -> String {
        let mut entries = Vec::new();
        if let Some(default) = slots.default {
            entries.push(cstr!(
                "{}: {}",
                quote_js("default"),
                self.block_slot_callback(default, "", indent)
            ));
        }
        entries.extend(slots.static_slots.iter().map(|slot| {
            cstr!(
                "{}: {}",
                object_key(self.plan, &slot.name),
                self.slot_callback(slot, indent)
            )
        }));
        if !slots.dynamic_slots.is_empty() {
            let dynamic = slots
                .dynamic_slots
                .iter()
                .map(|slot| cstr!("() => ({})", self.dynamic_slot(slot, indent)))
                .collect::<Vec<_>>()
                .join(", ");
            entries.push(cstr!("$: [{dynamic}]"));
        }
        cstr!("{{ {} }}", entries.join(", "))
    }

    fn slot_callback(&mut self, slot: &VaporSlot, indent: usize) -> String {
        let params = slot
            .bindings
            .iter()
            .map(|binding| binding.pattern.as_ref())
            .collect::<Vec<_>>()
            .join(", ");
        self.block_slot_callback(slot.body, &params, indent)
    }

    fn block_slot_callback(&mut self, body: VaporBlockId, params: &str, indent: usize) -> String {
        let callback = self.callback(body, params, indent);
        if self.block_needs_vapor_ctx(body) {
            use_helper(&mut self.helpers, "withVaporCtx");
            cstr!("_withVaporCtx({callback})")
        } else {
            callback
        }
    }

    fn block_needs_vapor_ctx(&self, body: VaporBlockId) -> bool {
        self.plan
            .block(body)
            .expect("validated Vapor slot block")
            .operations
            .iter()
            .any(|operation| match operation {
                VaporOperation::Component { .. } | VaporOperation::SlotOutlet { .. } => true,
                VaporOperation::Fragment { body, .. }
                | VaporOperation::Element { body, .. }
                | VaporOperation::SlotContent { body, .. }
                | VaporOperation::Iterate { body, .. } => self.block_needs_vapor_ctx(*body),
                VaporOperation::Conditional { branches, .. } => branches
                    .iter()
                    .any(|branch| self.block_needs_vapor_ctx(branch.body)),
                VaporOperation::StaticHtml { .. }
                | VaporOperation::DynamicText { .. }
                | VaporOperation::HoistRef { .. }
                | VaporOperation::Unsupported { .. } => false,
            })
    }

    fn dynamic_slot(&mut self, slot: &VaporDynamicSlot, indent: usize) -> String {
        match slot {
            VaporDynamicSlot::Direct(slot) => self.slot_descriptor(slot, indent),
            VaporDynamicSlot::Conditional { branches, .. } => {
                self.conditional_slot(branches, indent)
            }
            VaporDynamicSlot::Iterated {
                source,
                value,
                key,
                index,
                slot,
                ..
            } => {
                use_helper(&mut self.helpers, "createForSlots");
                let params = [Some(value), key.as_ref(), index.as_ref()]
                    .into_iter()
                    .flatten()
                    .map(|binding| binding.pattern.as_ref())
                    .collect::<Vec<_>>()
                    .join(", ");
                cstr!(
                    "_createForSlots({}, ({params}) => ({}))",
                    expression(self.plan, *source),
                    self.slot_descriptor(slot, indent + 1)
                )
            }
        }
    }

    fn conditional_slot(
        &mut self,
        branches: &[VaporConditionalSlotBranch],
        indent: usize,
    ) -> String {
        let Some((branch, rest)) = branches.split_first() else {
            return String::from("void 0");
        };
        let slot = branch.slot.as_ref().map_or_else(
            || String::from("void 0"),
            |slot| self.slot_descriptor(slot, indent),
        );
        let Some(condition) = branch.condition else {
            return slot;
        };
        let fallback = self.conditional_slot(rest, indent + 1);
        cstr!(
            "({}) ? {slot} : {fallback}",
            expression(self.plan, condition)
        )
    }

    fn slot_descriptor(&mut self, slot: &VaporSlot, indent: usize) -> String {
        cstr!(
            "{{ name: {}, fn: {} }}",
            name(self.plan, &slot.name),
            self.slot_callback(slot, indent)
        )
    }
}

fn dynamic_component_value(plan: &super::super::VaporPlan, properties: &[VaporProperty]) -> String {
    for property in properties {
        match property {
            VaporProperty::Attribute {
                name: VaporName::Static(name),
                value,
                ..
            } if name.as_ref() == "is" => {
                return match value {
                    None => String::from("null"),
                    Some(super::super::VaporAttributeValue::Static(value)) => quote_js(value),
                    Some(super::super::VaporAttributeValue::Expression(value)) => {
                        expression(plan, *value).to_compact_string()
                    }
                };
            }
            VaporProperty::Directive(directive)
                if directive.name.as_ref() == "bind"
                    && matches!(&directive.argument, Some(VaporName::Static(name)) if name.as_ref() == "is") =>
            {
                if let Some(value) = directive.expression {
                    return expression(plan, value).to_compact_string();
                }
            }
            _ => {}
        }
    }
    String::from("null")
}
