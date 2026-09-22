//! The cross-file rules' declared fact demands (Davinci P4-3).
//!
//! Every cross-file rule that reads a Croquis fact group names its demand
//! here as const data, so the debug detector (TS-35) refuses any read
//! outside it and the demand set of the whole lane reads in one place.

use vize_croquis::facts::{Bindings, Demand, FactConsumer, FactGroup};

/// `cross-file/error-boundary`: script bindings (`onErrorCaptured`).
pub struct ErrorBoundaryRule;

impl FactConsumer for ErrorBoundaryRule {
    const NAME: &'static str = "cross-file/error-boundary";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

/// `cross-file/component-resolution`: script bindings that register a
/// template component locally.
pub struct ComponentResolutionRule;

impl FactConsumer for ComponentResolutionRule {
    const NAME: &'static str = "cross-file/component-resolution";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}
