//! Explicit complete original Vue 2 scriptless SFC documents.

mod build;
mod observation;
mod options;
mod refusal;

pub use build::observe_native_vue2_sfc_in;
pub use observation::NativeVue2SfcObservation;
pub use options::NativeVue2SfcOptions;
pub use refusal::{NativeVue2SfcNode, NativeVue2SfcRefusal};

#[cfg(test)]
mod tests;
