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
    IS_ANY => ("$IsAny", "IsAny"),
    COMPONENT_NAVIGATION_PROPS => ("$ComponentNavigationProps", "$ComponentNavigationProps"),
    EMIT_LISTENERS => ("$EmitListeners", "$EmitListeners"),
    REQUIRED_FALLTHROUGH_PROPS => ("$RequiredFallthroughProps", "$RequiredFallthroughProps"),
    COMPONENT_RAW_PROPS => ("$ComponentRawProps", "$ComponentRawProps"),
    FALLTHROUGH_VALUE => ("$FallthroughValue", "$FallthroughValue"),
    COMPONENT_EVENTS => ("$ComponentEvents", "$ComponentEvents"),
    IS_LIBRARY_GENERIC_EXPORT => ("$IsLibraryGenericExport", "$IsLibraryGenericExport"),
    PROP_CHECKER => ("$PropChecker", "$PropChecker"),
    PROP_VALUE => ("$PropValue", "$PropValue"),
    FALLTHROUGH_SPREAD_CHECKER => ("$FallthroughSpreadChecker", "$FallthroughSpreadChecker"),
    CALLABLE_PROP => ("$CallableProp", "$CallableProp"),
    PROPS_RESOLVER => ("$PropsResolver", "$PropsResolver"),
    RESOLVED_CONTEXT_PROPS => ("$ResolvedContextProps", "$ResolvedContextProps"),
    PROPS_SELECTOR => ("$PropsSelector", "$PropsSelector"),
    RESOLVED_PROP => ("$ResolvedProp", "$ResolvedProp"),
    COMPONENT_REF_CALLBACK => ("$ComponentRefCallback", "$ComponentRefCallback"),
    STRUCTURAL_SLOTS => ("$StructuralSlots", "$StructuralSlots"),
    NAVIGATION_SLOTS => ("$NavigationSlots", "$NavigationSlots"),
    REQUIRED_SLOTS => ("$RequiredSlots", "$RequiredSlots"),
    SLOT_CHILDREN => ("$SlotChildren", "$SlotChildren"),
    PROVIDED_SLOT_CHILDREN => ("$ProvidedSlotChildren", "$ProvidedSlotChildren"),
    SLOT_HOST_SLOTS => ("$SlotHostSlots", "$SlotHostSlots"),
    SLOTS_FACTORY => ("$SlotsFactory", "$SlotsFactory"),
    SLOT_CONSTRUCTOR => ("$SlotConstructor", "$SlotConstructor"),
    SLOT_CONSTRUCTOR_WITHOUT_ARGUMENTS => ("$SlotConstructorWithoutArguments", "$SlotConstructorWithoutArguments"),
    SLOT_CONSTRUCTOR_WITH_ARGUMENTS => ("$SlotConstructorWithArguments", "$SlotConstructorWithArguments"),
    SLOT_CONSTRUCTOR_INSTANCE => ("$SlotConstructorInstance", "$SlotConstructorInstance"),
    SLOT_CONSTRUCTOR_ARGS => ("$SlotConstructorArgs", "$SlotConstructorArgs"),
    SLOT_HOST_SELECTOR => ("$SlotHostSelector", "$SlotHostSelector"),
    SLOT_PAYLOAD => ("$SlotPayload", "$SlotPayload"),
    EMIT_OWNED_KEYS => ("$EmitOwnedKeys", "$EmitOwnedKeys"),
    EMIT_PROPS_FACTORY => ("$EmitPropsFactory", "$EmitPropsFactory"),
    EMIT_PROPS => ("__EmitProps", "__EmitProps"),
}
