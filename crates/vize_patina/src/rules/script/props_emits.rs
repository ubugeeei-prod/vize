//! Props & emits validation script rules.
//!
//! Groups the rule and shared-helper modules that analyze Vue component
//! `defineProps`/`defineEmits` declarations and their Options API equivalents,
//! so the parent `script` module file does not grow per added rule.

mod emits_source;
mod no_required_prop_with_default;
mod no_reserved_props;
mod no_unused_emit_declarations;
mod props_source;
mod require_default_prop;
mod require_explicit_emits;
mod require_prop_types;
mod require_typed_object_prop;
mod require_valid_default_prop;
mod return_in_emits_validator;

pub use no_required_prop_with_default::NoRequiredPropWithDefault;
pub use no_reserved_props::NoReservedProps;
pub use no_unused_emit_declarations::NoUnusedEmitDeclarations;
pub use require_default_prop::RequireDefaultProp;
pub use require_explicit_emits::RequireExplicitEmits;
pub use require_prop_types::RequirePropTypes;
pub use require_typed_object_prop::RequireTypedObjectProp;
pub use require_valid_default_prop::RequireValidDefaultProp;
pub use return_in_emits_validator::ReturnInEmitsValidator;
