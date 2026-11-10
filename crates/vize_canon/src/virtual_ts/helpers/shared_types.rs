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
        /// Public SDK exports consumed by native shared virtual modules.
        pub const SHARED_VIRTUAL_TYPE_EXPORTS: &[&str] = &[$($exported),+];
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
    COMPONENT_RAW_PROPS => ("$ComponentRawProps", "$ComponentRawProps"),
    COMPONENT_EVENTS => ("$ComponentEvents", "$ComponentEvents"),
    IS_LIBRARY_GENERIC_EXPORT => ("$IsLibraryGenericExport", "$IsLibraryGenericExport"),
    USAGE_PROPS => ("$UsageProps", "$UsageProps"),
    USAGE_REQUIRED_FALLTHROUGH => ("$UsageRequiredFallthrough", "$UsageRequiredFallthrough"),
    USAGE_LIBRARY => ("$UsageLibrary", "$UsageLibrary"),
    USAGE_PROPS_BOX => ("$UsagePropsBox", "$UsagePropsBox"),
    USAGE_EMIT_PROPS_BOX => ("$UsageEmitPropsBox", "$UsageEmitPropsBox"),
    USAGE_NATIVE_FALLTHROUGH_BOX => ("$UsageNativeFallthroughBox", "$UsageNativeFallthroughBox"),
    PROP_CHECKER => ("$PropChecker", "$PropChecker"),
    PROP_VALUE => ("$PropValue", "$PropValue"),
    PROP_VALUE_OR => ("$PropValueOr", "$PropValueOr"),
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
    REGISTERED_SLOT_PROVIDER => ("$RegisteredSlotProvider", "$RegisteredSlotProvider"),
    EMIT_OWNED_KEYS => ("$EmitOwnedKeys", "$EmitOwnedKeys"),
    LISTENER_ARGS => ("$ListenerArgs", "$ListenerArgs"),
    LISTENER_CONTRACT => ("$ListenerContract", "$ListenerContract"),
    EMIT_PROPS_FACTORY => ("$EmitPropsFactory", "$EmitPropsFactory"),
    CLOSED_EMIT_TARGET => ("__vize_emit_closed", "$ClosedEmitTarget"),
    CLOSED_EMIT_FACTORY => ("__vize_emit_closed_factory", "$ClosedEmitFactory"),
    EMIT_PROPS => ("__EmitProps", "__EmitProps"),
    TEMPLATE_REF_UNWRAP => ("__U", "$TemplateRefUnwrap"),
    TEMPLATE_REF_UNWRAP_NARROWED => ("__UN", "$TemplateRefUnwrapNarrowed"),
}
