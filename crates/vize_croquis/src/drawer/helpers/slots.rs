//! Slot props and inline callback parameter extraction.
//!
//! Handles parsing of:
//! - `v-slot` directive patterns (e.g., `v-slot="{ item, index }"`)
//! - Inline callback parameters from arrow functions and function expressions
//!   used in event handlers (e.g., `@click="(e) => handle(e)"`)

mod callbacks;
mod slot_props;

pub use callbacks::extract_inline_callback_params;
pub use slot_props::extract_slot_props;
