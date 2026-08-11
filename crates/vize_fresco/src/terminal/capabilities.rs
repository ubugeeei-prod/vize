//! Deterministic terminal capability resolution and presentation fallbacks.

mod accessors;
mod model;
mod resolve;
mod style;

#[cfg(test)]
mod tests;

pub use model::{
    CapabilityDecision, CapabilityReason, ColorPreference, ColorSupport, FeaturePreference,
    TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};
