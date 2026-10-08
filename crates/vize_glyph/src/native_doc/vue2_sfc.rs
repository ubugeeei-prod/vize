//! Explicit complete original Vue 2 scriptless SFC documents.

#[path = "vue2_sfc/build.rs"]
mod build;
#[path = "vue2_sfc/observation.rs"]
mod observation;
#[path = "vue2_sfc/options.rs"]
mod options;
#[path = "vue2_sfc/refusal.rs"]
mod refusal;

pub use build::observe_native_vue2_sfc_in;
pub use observation::NativeVue2SfcObservation;
pub use options::NativeVue2SfcOptions;
pub use refusal::{NativeVue2SfcNode, NativeVue2SfcRefusal};

#[cfg(test)]
#[path = "vue2_sfc/tests.rs"]
mod tests;
