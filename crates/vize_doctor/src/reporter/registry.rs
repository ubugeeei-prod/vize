//! Explicitly owned and deterministically ordered reporter registration.

use std::{collections::BTreeMap, error::Error, fmt};

use vize_s0::String;

use super::{DoctorReporter, ReporterContractError, ReporterDescriptor};

/// Registration failure for an explicitly owned reporter set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReporterRegistrationError {
    /// The reporter descriptor was invalid.
    InvalidContract(ReporterContractError),
    /// Another reporter already owns the stable identifier.
    DuplicateId(String),
}

impl fmt::Display for ReporterRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContract(error) => error.fmt(formatter),
            Self::DuplicateId(id) => write!(formatter, "reporter id {id} is already registered"),
        }
    }
}

impl Error for ReporterRegistrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidContract(error) => Some(error),
            Self::DuplicateId(_) => None,
        }
    }
}

/// Deterministically ordered, explicitly owned reporter registry.
///
/// Construct one set per application, editor session, CI run, or integration.
/// Registration never mutates process-global state, so independent consumers
/// cannot replace each other's reporters.
#[derive(Default)]
pub struct ReporterSet {
    reporters: BTreeMap<String, Box<dyn DoctorReporter>>,
}

impl ReporterSet {
    /// Creates an empty reporter set.
    pub const fn new() -> Self {
        Self {
            reporters: BTreeMap::new(),
        }
    }

    /// Registers one reporter after validating its descriptor.
    pub fn register(
        &mut self,
        reporter: impl DoctorReporter + 'static,
    ) -> Result<(), ReporterRegistrationError> {
        self.register_boxed(Box::new(reporter))
    }

    /// Registers a dynamically selected reporter after validating its descriptor.
    ///
    /// This is the object-safe entry point for integrations that load reporters
    /// from configuration or another explicit application-owned catalog.
    pub fn register_boxed(
        &mut self,
        reporter: Box<dyn DoctorReporter>,
    ) -> Result<(), ReporterRegistrationError> {
        reporter
            .descriptor()
            .validate()
            .map_err(ReporterRegistrationError::InvalidContract)?;
        let id: String = reporter.descriptor().id().into();
        if self.reporters.contains_key(&id) {
            return Err(ReporterRegistrationError::DuplicateId(id));
        }
        self.reporters.insert(id, reporter);
        Ok(())
    }

    /// Returns a reporter by stable identifier.
    pub fn get(&self, id: &str) -> Option<&dyn DoctorReporter> {
        self.reporters.get(id).map(Box::as_ref)
    }

    /// Returns descriptors in stable identifier order.
    pub fn descriptors(&self) -> impl ExactSizeIterator<Item = &ReporterDescriptor> {
        self.reporters
            .values()
            .map(|reporter| reporter.descriptor())
    }

    /// Returns the number of registered reporters.
    pub fn len(&self) -> usize {
        self.reporters.len()
    }

    /// Returns whether no reporters are registered.
    pub fn is_empty(&self) -> bool {
        self.reporters.is_empty()
    }
}
