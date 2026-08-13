//! Deterministic terminal capability resolution and presentation fallbacks.
//!
//! Capability resolution produces values together with stable reasons, so
//! diagnostic tools can document and snapshot why a presentation was downgraded
//! or disabled. The resolver recognizes explicit preferences, `FORCE_COLOR`,
//! `NO_COLOR`, `CLICOLOR`, `CLICOLOR_FORCE`, redirected output, `TERM=dumb`,
//! UTF-8 and non-UTF-8 locales, `FRESCO_UNICODE`, `FRESCO_INTERACTIVE`, and
//! `CI`.
//!
//! Defaults are conservative: color, Unicode, and interactivity are automatic;
//! redirected output and dumb terminals are non-interactive; CI disables
//! automatic interactivity; widths below 60 cells are flagged as narrow; and a
//! zero narrow width disables narrow-layout detection. Invalid Fresco boolean
//! overrides fail closed with
//! [`CapabilityReason::InvalidEnvironmentOverride`].

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
