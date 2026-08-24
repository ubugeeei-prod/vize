use serde::Serialize;
use vize_carton::String;

use super::CapabilityInvalidation;

/// Machine-readable summary of why a previous capability result was or was not reusable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityInvalidationTelemetry {
    reusable: bool,
    capability_changed: bool,
    implementation_changed: bool,
    configuration_changed: bool,
    added_input_count: usize,
    removed_input_count: usize,
    changed_input_count: usize,
    added_inputs: Vec<String>,
    removed_inputs: Vec<String>,
    changed_inputs: Vec<String>,
}

impl CapabilityInvalidationTelemetry {
    pub(super) fn from_invalidation(invalidation: &CapabilityInvalidation) -> Self {
        Self {
            reusable: invalidation.is_reusable(),
            capability_changed: invalidation.capability_changed(),
            implementation_changed: invalidation.implementation_changed(),
            configuration_changed: invalidation.configuration_changed(),
            added_input_count: invalidation.added_inputs().len(),
            removed_input_count: invalidation.removed_inputs().len(),
            changed_input_count: invalidation.changed_inputs().len(),
            added_inputs: invalidation.added_inputs().to_vec(),
            removed_inputs: invalidation.removed_inputs().to_vec(),
            changed_inputs: invalidation.changed_inputs().to_vec(),
        }
    }

    /// Returns whether all cache identity boundaries are unchanged.
    pub const fn is_reusable(&self) -> bool {
        self.reusable
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

    /// Returns the number of newly declared invalidation inputs.
    pub const fn added_input_count(&self) -> usize {
        self.added_input_count
    }

    /// Returns the number of no-longer-declared invalidation inputs.
    pub const fn removed_input_count(&self) -> usize {
        self.removed_input_count
    }

    /// Returns the number of content-changed invalidation inputs.
    pub const fn changed_input_count(&self) -> usize {
        self.changed_input_count
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

impl CapabilityInvalidation {
    /// Returns deterministic telemetry for this cache invalidation decision.
    #[must_use]
    pub fn telemetry(&self) -> CapabilityInvalidationTelemetry {
        CapabilityInvalidationTelemetry::from_invalidation(self)
    }
}
