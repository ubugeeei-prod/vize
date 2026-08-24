//! Slot generation functions.
//!
//! Generates slot objects for component children.

mod create_slots;
mod detect;
mod generate;
mod name;
mod outlet;
mod params;

#[cfg(test)]
mod tests;

pub(crate) use outlet::{
    generate_slot_outlet_name, generate_slot_outlet_props, generate_slot_outlet_props_with_key,
    has_slot_outlet_props,
};

pub use detect::{has_dynamic_slots_flag, has_slot_children, needs_dynamic_slots_patch};
pub use generate::generate_slots;
