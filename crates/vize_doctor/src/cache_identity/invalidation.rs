use std::cmp::Ordering;

use serde::Serialize;
use vize_s0::String;

use super::CapabilityCacheIdentity;

mod telemetry;

pub use telemetry::CapabilityInvalidationTelemetry;

/// Exact reasons a previous capability result cannot be reused.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityInvalidation {
    pub(super) capability_changed: bool,
    pub(super) implementation_changed: bool,
    pub(super) configuration_changed: bool,
    pub(super) added_inputs: Vec<String>,
    pub(super) removed_inputs: Vec<String>,
    pub(super) changed_inputs: Vec<String>,
}

impl CapabilityInvalidation {
    /// Returns whether the previous result is exactly reusable.
    pub fn is_reusable(&self) -> bool {
        !self.capability_changed
            && !self.implementation_changed
            && !self.configuration_changed
            && self.added_inputs.is_empty()
            && self.removed_inputs.is_empty()
            && self.changed_inputs.is_empty()
    }

    /// Returns whether the stable capability identifier changed.
    pub const fn capability_changed(&self) -> bool {
        self.capability_changed
    }

    /// Returns whether analyzer implementation behavior changed.
    pub const fn implementation_changed(&self) -> bool {
        self.implementation_changed
    }

    /// Returns whether behavior-affecting configuration changed.
    pub const fn configuration_changed(&self) -> bool {
        self.configuration_changed
    }

    /// Returns newly declared inputs in stable order.
    pub fn added_inputs(&self) -> &[String] {
        &self.added_inputs
    }

    /// Returns no-longer-declared inputs in stable order.
    pub fn removed_inputs(&self) -> &[String] {
        &self.removed_inputs
    }

    /// Returns content-changed inputs in stable order.
    pub fn changed_inputs(&self) -> &[String] {
        &self.changed_inputs
    }
}

impl CapabilityCacheIdentity {
    /// Compares this current identity with a previous analysis run.
    ///
    /// Input changes are classified in one linear merge over the two canonical
    /// input lists. The result is reusable only when every boundary is exact.
    #[must_use]
    pub fn invalidation_from(&self, previous: &Self) -> CapabilityInvalidation {
        let mut invalidation = CapabilityInvalidation {
            capability_changed: self.capability != previous.capability,
            implementation_changed: self.implementation_fingerprint
                != previous.implementation_fingerprint,
            configuration_changed: self.configuration_fingerprint
                != previous.configuration_fingerprint,
            ..CapabilityInvalidation::default()
        };
        let mut previous_inputs = previous.inputs.iter().peekable();
        let mut current_inputs = self.inputs.iter().peekable();

        loop {
            match (previous_inputs.peek(), current_inputs.peek()) {
                (Some(previous), Some(current)) => match previous.id.cmp(&current.id) {
                    Ordering::Less => {
                        invalidation.removed_inputs.push(previous.id.clone());
                        previous_inputs.next();
                    }
                    Ordering::Greater => {
                        invalidation.added_inputs.push(current.id.clone());
                        current_inputs.next();
                    }
                    Ordering::Equal => {
                        if previous.fingerprint != current.fingerprint {
                            invalidation.changed_inputs.push(current.id.clone());
                        }
                        previous_inputs.next();
                        current_inputs.next();
                    }
                },
                (Some(previous), None) => {
                    invalidation.removed_inputs.push(previous.id.clone());
                    previous_inputs.next();
                }
                (None, Some(current)) => {
                    invalidation.added_inputs.push(current.id.clone());
                    current_inputs.next();
                }
                (None, None) => break,
            }
        }
        invalidation
    }
}
