//! Complete original scriptless Vue SFC observations and native formatting.

#[path = "sfc/build.rs"]
mod build;
#[path = "sfc/observation.rs"]
mod observation;
#[path = "sfc/options.rs"]
mod options;
#[path = "sfc/refusal.rs"]
mod refusal;

pub use build::observe_native_sfc_in;
pub use observation::NativeSfcObservation;
pub use options::{NativeSfcDirectivePolicy, NativeSfcOptions};
pub use refusal::{NativeSfcBlockRole, NativeSfcRefusal};
