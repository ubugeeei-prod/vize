//! Complete original scriptless Vue SFC observations and native formatting.

mod build;
mod observation;
mod options;
mod refusal;

pub use build::observe_native_sfc_in;
pub use observation::NativeSfcObservation;
pub use options::{NativeSfcDirectivePolicy, NativeSfcOptions};
pub use refusal::{NativeSfcBlockRole, NativeSfcRefusal};
