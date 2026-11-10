//! Vue-template-specific compatibility rules.

mod no_literals_in_template;
mod no_restricted_v_bind;
mod prefer_v_model;
mod shared;
mod v_if_else_key;

pub use no_literals_in_template::NoLiteralsInTemplate;
pub use no_restricted_v_bind::NoRestrictedVBind;
pub use prefer_v_model::PreferVModel;
pub use v_if_else_key::VIfElseKey;
