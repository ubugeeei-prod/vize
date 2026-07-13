//! Props type, artifact, and template-binding generation.

mod declarations;
mod generics;
mod setup_scoped;
mod template_bindings;
mod template_names;
mod variables;
mod with_defaults;

use super::helpers::{is_reserved_identifier, to_safe_identifier};
pub(super) use declarations::append_model_props_type_literal;
pub(crate) use declarations::{OptionsApiPropsSource, generate_props_type};
pub(crate) use generics::{
    add_generic_defaults, extract_generic_names, strip_const_modifiers, strip_outer_angle_brackets,
    type_reference_lookup_key,
};
use setup_scoped::props_type_ref;
pub(crate) use setup_scoped::{PropsTypeEmission, generate_setup_scoped_props_artifact};
pub(crate) use template_names::collect_template_prop_names;
pub(crate) use variables::generate_props_variables;
