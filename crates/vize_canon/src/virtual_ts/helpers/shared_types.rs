//! Collision-proof references to helpers shared by batch-generated modules.
//!
//! An authored `<script setup>` type alias lives between the module preamble
//! and the nested generated template function. Unqualified ambient helper
//! names would therefore resolve to the authored alias. The batch path reaches
//! the canonical declarations through an ambient external module instead; an
//! import type expression has no lexical identifier for authored code to
//! capture. Standalone virtual documents keep their existing local helpers.

#[derive(Clone, Copy)]
pub(crate) struct SharedTypeRef {
    embedded: &'static str,
    hoisted: &'static str,
}

impl SharedTypeRef {
    pub(crate) const fn select(self, hoisted: bool) -> &'static str {
        if hoisted { self.hoisted } else { self.embedded }
    }
}

macro_rules! shared_type_refs {
    ($($name:ident => ($embedded:literal, $exported:literal)),+ $(,)?) => {
        $(
            pub(crate) const $name: SharedTypeRef = SharedTypeRef {
                embedded: $embedded,
                hoisted: concat!("import(\"vize:virtual-ts\").", $exported),
            };
        )+
    };
}

shared_type_refs! {
    IS_ANY => ("__VizeIsAny", "IsAny"),
    COMPONENT_NAVIGATION_PROPS => ("__VizeComponentNavigationProps", "__VizeComponentNavigationProps"),
    EMIT_LISTENERS => ("__VizeEmitListeners", "__VizeEmitListeners"),
    REQUIRED_FALLTHROUGH_PROPS => ("__VizeRequiredFallthroughProps", "__VizeRequiredFallthroughProps"),
    COMPONENT_RAW_PROPS => ("__VizeComponentRawProps", "__VizeComponentRawProps"),
    FALLTHROUGH_VALUE => ("__VizeFallthroughValue", "__VizeFallthroughValue"),
    COMPONENT_EVENTS => ("__VizeComponentEvents", "__VizeComponentEvents"),
    IS_LIBRARY_GENERIC_EXPORT => ("__VizeIsLibraryGenericExport", "__VizeIsLibraryGenericExport"),
    PROP_CHECKER => ("__VizePropChecker", "__VizePropChecker"),
    PROP_VALUE => ("__VizePropValue", "__VizePropValue"),
    FALLTHROUGH_SPREAD_CHECKER => ("__VizeFallthroughSpreadChecker", "__VizeFallthroughSpreadChecker"),
    CALLABLE_PROP => ("__VizeCallableProp", "__VizeCallableProp"),
    PROPS_RESOLVER => ("__VizePropsResolver", "__VizePropsResolver"),
    RESOLVED_CONTEXT_PROPS => ("__VizeResolvedContextProps", "__VizeResolvedContextProps"),
    PROPS_SELECTOR => ("__VizePropsSelector", "__VizePropsSelector"),
    RESOLVED_PROP => ("__VizeResolvedProp", "__VizeResolvedProp"),
    COMPONENT_REF_CALLBACK => ("__VizeComponentRefCallback", "__VizeComponentRefCallback"),
    STRUCTURAL_SLOTS => ("__VizeStructuralSlots", "__VizeStructuralSlots"),
    NAVIGATION_SLOTS => ("__VizeNavigationSlots", "__VizeNavigationSlots"),
    REQUIRED_SLOTS => ("__VizeRequiredSlots", "__VizeRequiredSlots"),
    SLOT_CHILDREN => ("__VizeSlotChildren", "__VizeSlotChildren"),
    PROVIDED_SLOT_CHILDREN => ("__VizeProvidedSlotChildren", "__VizeProvidedSlotChildren"),
    SLOT_HOST_SLOTS => ("__VizeSlotHostSlots", "__VizeSlotHostSlots"),
    SLOTS_FACTORY => ("__VizeSlotsFactory", "__VizeSlotsFactory"),
    SLOT_CONSTRUCTOR => ("__VizeSlotConstructor", "__VizeSlotConstructor"),
    SLOT_CONSTRUCTOR_WITHOUT_ARGUMENTS => ("__VizeSlotConstructorWithoutArguments", "__VizeSlotConstructorWithoutArguments"),
    SLOT_CONSTRUCTOR_WITH_ARGUMENTS => ("__VizeSlotConstructorWithArguments", "__VizeSlotConstructorWithArguments"),
    SLOT_CONSTRUCTOR_INSTANCE => ("__VizeSlotConstructorInstance", "__VizeSlotConstructorInstance"),
    SLOT_CONSTRUCTOR_ARGS => ("__VizeSlotConstructorArgs", "__VizeSlotConstructorArgs"),
    SLOT_HOST_SELECTOR => ("__VizeSlotHostSelector", "__VizeSlotHostSelector"),
    SLOT_PAYLOAD => ("__VizeSlotPayload", "__VizeSlotPayload"),
    EMIT_OWNED_KEYS => ("__VizeEmitOwnedKeys", "__VizeEmitOwnedKeys"),
    EMIT_PROPS_FACTORY => ("__VizeEmitPropsFactory", "__VizeEmitPropsFactory"),
    EMIT_PROPS => ("__EmitProps", "__EmitProps"),
}
