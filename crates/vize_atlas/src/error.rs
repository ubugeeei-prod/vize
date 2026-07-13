//! Errors produced while registering, planning, and executing products.

mod plan;

pub use plan::PlanError;

use std::{error::Error, fmt};
use vize_carton::String;

use crate::{
    InputId, ProductId, ProductRequest, ProviderId, SourceId, SourceInputId, SourceRevision,
};

/// A provider could not be registered.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RegisterProviderError {
    /// The same concrete provider type was registered more than once.
    DuplicateProvider {
        provider: ProviderId,
        product: ProductId,
    },
    /// The registry generation counter cannot advance safely.
    ProviderGenerationExhausted,
}

impl fmt::Display for RegisterProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateProvider { provider, product } => {
                write!(
                    formatter,
                    "provider {provider} is already registered for {product}"
                )
            }
            Self::ProviderGenerationExhausted => {
                formatter.write_str("provider registry generation is exhausted")
            }
        }
    }
}

impl Error for RegisterProviderError {}

/// A provider failed or violated its declared dependency boundary.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProviderError {
    /// Provider-specific failure text.
    Message(String),
    /// A provider queried a product absent from its dependency declaration.
    UndeclaredDependency {
        provider: ProviderId,
        dependency: ProductId,
    },
    /// A declared dependency was not resolved before its consumer.
    DependencyUnavailable {
        provider: ProviderId,
        dependency: ProductId,
    },
    /// An erased dependency value did not match its typed product contract.
    DependencyTypeMismatch(ProductId),
    /// A provider queried an undeclared cross-source request.
    UndeclaredRequest {
        provider: ProviderId,
        dependency: ProductRequest,
    },
    /// A declared cross-source dependency was unavailable.
    RequestUnavailable {
        provider: ProviderId,
        dependency: ProductRequest,
    },
    /// An erased cross-source value did not match its typed contract.
    RequestTypeMismatch(ProductRequest),
}

impl ProviderError {
    /// Build a provider-specific error message.
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => formatter.write_str(message),
            Self::UndeclaredDependency {
                provider,
                dependency,
            } => write!(
                formatter,
                "provider {provider} queried undeclared dependency {dependency}"
            ),
            Self::DependencyUnavailable {
                provider,
                dependency,
            } => write!(
                formatter,
                "dependency {dependency} was unavailable while executing {provider}"
            ),
            Self::DependencyTypeMismatch(product) => {
                write!(
                    formatter,
                    "cached value for {product} has the wrong concrete type"
                )
            }
            Self::UndeclaredRequest {
                provider,
                dependency,
            } => write!(
                formatter,
                "provider {provider} queried undeclared request {dependency}"
            ),
            Self::RequestUnavailable {
                provider,
                dependency,
            } => write!(
                formatter,
                "request {dependency} was unavailable while executing {provider}"
            ),
            Self::RequestTypeMismatch(request) => {
                write!(
                    formatter,
                    "cached value for {request} has the wrong concrete type"
                )
            }
        }
    }
}

impl Error for ProviderError {}

/// Planning or executing a typed query failed.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueryError {
    /// Dependency planning failed before any provider executed.
    Plan(PlanError),
    /// A source changed after its plan was created.
    StaleSourcePlan {
        source: SourceId,
        planned: SourceRevision,
        current: SourceRevision,
    },
    /// The provider registry changed after this plan was created.
    StaleProviderPlan { planned: u64, current: u64 },
    /// A typed compilation input changed after this plan was created.
    StaleInputPlan {
        input: InputId,
        planned: u64,
        current: u64,
    },
    /// A typed source input changed after this plan was created.
    StaleSourceInputPlan {
        source: SourceId,
        input: SourceInputId,
        planned: u64,
        current: u64,
    },
    /// One provider invocation failed.
    ProviderFailed {
        source: SourceId,
        product: ProductId,
        provider: ProviderId,
        error: Box<ProviderError>,
    },
    /// The typed root was unexpectedly absent from a completed outcome.
    MissingProduct(ProductId),
    /// An erased cached value did not match its typed product contract.
    ProductTypeMismatch(ProductId),
    /// A typed product request was absent from a completed outcome.
    MissingRequest(ProductRequest),
    /// A cross-source outcome value did not match its typed contract.
    RequestTypeMismatch(ProductRequest),
}

impl From<PlanError> for QueryError {
    fn from(error: PlanError) -> Self {
        Self::Plan(error)
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plan(error) => error.fmt(formatter),
            Self::StaleSourcePlan {
                source,
                planned,
                current,
            } => write!(
                formatter,
                "plan for {source} was built at {planned}, but the source is now {current}"
            ),
            Self::StaleProviderPlan { planned, current } => write!(
                formatter,
                "plan used provider generation {planned}, but the registry is now {current}"
            ),
            Self::StaleInputPlan {
                input,
                planned,
                current,
            } => write!(
                formatter,
                "plan used input {input} at revision {planned}, but it is now {current}"
            ),
            Self::StaleSourceInputPlan {
                source,
                input,
                planned,
                current,
            } => write!(
                formatter,
                "plan used source input {input} for {source} at revision {planned}, but it is now {current}"
            ),
            Self::ProviderFailed {
                source,
                product,
                provider,
                error,
            } => write!(
                formatter,
                "provider {provider} for {product} failed for {source}: {error}"
            ),
            Self::MissingProduct(product) => {
                write!(formatter, "completed outcome does not contain {product}")
            }
            Self::ProductTypeMismatch(product) => {
                write!(
                    formatter,
                    "outcome value for {product} has the wrong concrete type"
                )
            }
            Self::MissingRequest(request) => {
                write!(
                    formatter,
                    "completed outcome does not contain request {request}"
                )
            }
            Self::RequestTypeMismatch(request) => {
                write!(
                    formatter,
                    "outcome value for {request} has the wrong concrete type"
                )
            }
        }
    }
}

impl Error for QueryError {}
