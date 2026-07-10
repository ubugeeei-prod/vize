use std::{error::Error, fmt};

use crate::{ProductId, ProductRequest, ProviderId, SourceId, SourceRevision};

/// A requested dependency closure could not be planned.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlanError {
    SourceNotFound(SourceId),
    StaleEmbeddedSource {
        source: SourceId,
        parent: SourceId,
        recorded: SourceRevision,
        current: SourceRevision,
    },
    NoRoots,
    MissingProvider {
        product: ProductId,
        required_by: Option<ProductId>,
    },
    NoApplicableProvider {
        product: ProductId,
        required_by: Option<ProductId>,
        registered: Vec<ProviderId>,
    },
    AmbiguousProvider {
        product: ProductId,
        required_by: Option<ProductId>,
        applicable: Vec<ProviderId>,
    },
    DependencyCycle {
        path: Vec<ProductId>,
    },
    MissingRequestProvider {
        request: ProductRequest,
        required_by: Option<ProductRequest>,
    },
    NoApplicableRequestProvider {
        request: ProductRequest,
        required_by: Option<ProductRequest>,
        registered: Vec<ProviderId>,
    },
    AmbiguousRequestProvider {
        request: ProductRequest,
        required_by: Option<ProductRequest>,
        applicable: Vec<ProviderId>,
    },
    RequestDependencyCycle {
        path: Vec<ProductRequest>,
    },
}

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNotFound(source) => write!(formatter, "source {source} does not exist"),
            Self::StaleEmbeddedSource {
                source,
                parent,
                recorded,
                current,
            } => write!(
                formatter,
                "embedded source {source} records {parent} at {recorded}, but it is now {current}"
            ),
            Self::NoRoots => formatter.write_str("an execution plan needs at least one root"),
            Self::MissingProvider {
                product,
                required_by: Some(parent),
            } => write!(
                formatter,
                "no provider is registered for {product}, required by {parent}"
            ),
            Self::MissingProvider {
                product,
                required_by: None,
            } => write!(
                formatter,
                "no provider is registered for requested product {product}"
            ),
            Self::NoApplicableProvider {
                product,
                required_by,
                registered,
            } => {
                write!(formatter, "no provider for {product} supports this context")?;
                write_requirement(formatter, *required_by)?;
                write_provider_list(formatter, "registered", registered)
            }
            Self::AmbiguousProvider {
                product,
                required_by,
                applicable,
            } => {
                write!(
                    formatter,
                    "multiple providers for {product} support this context"
                )?;
                write_requirement(formatter, *required_by)?;
                write_provider_list(formatter, "applicable", applicable)
            }
            Self::DependencyCycle { path } => {
                write_path(formatter, "provider dependency cycle: ", path)
            }
            Self::MissingRequestProvider {
                request,
                required_by,
            } => {
                write!(formatter, "no provider is registered for request {request}")?;
                write_request_requirement(formatter, *required_by)
            }
            Self::NoApplicableRequestProvider {
                request,
                required_by,
                registered,
            } => {
                write!(formatter, "no provider supports request {request}")?;
                write_request_requirement(formatter, *required_by)?;
                write_provider_list(formatter, "registered", registered)
            }
            Self::AmbiguousRequestProvider {
                request,
                required_by,
                applicable,
            } => {
                write!(formatter, "multiple providers support request {request}")?;
                write_request_requirement(formatter, *required_by)?;
                write_provider_list(formatter, "applicable", applicable)
            }
            Self::RequestDependencyCycle { path } => {
                write_path(formatter, "provider request dependency cycle: ", path)
            }
        }
    }
}

impl Error for PlanError {}

fn write_requirement(
    formatter: &mut fmt::Formatter<'_>,
    required_by: Option<ProductId>,
) -> fmt::Result {
    required_by.map_or(Ok(()), |product| {
        write!(formatter, ", required by {product}")
    })
}

fn write_request_requirement(
    formatter: &mut fmt::Formatter<'_>,
    required_by: Option<ProductRequest>,
) -> fmt::Result {
    required_by.map_or(Ok(()), |request| {
        write!(formatter, ", required by {request}")
    })
}

fn write_provider_list(
    formatter: &mut fmt::Formatter<'_>,
    label: &str,
    providers: &[ProviderId],
) -> fmt::Result {
    write!(formatter, "; {label}: [")?;
    for (index, provider) in providers.iter().enumerate() {
        if index > 0 {
            formatter.write_str(", ")?;
        }
        write!(formatter, "{provider}")?;
    }
    formatter.write_str("]")
}

fn write_path<T: fmt::Display>(
    formatter: &mut fmt::Formatter<'_>,
    prefix: &str,
    path: &[T],
) -> fmt::Result {
    formatter.write_str(prefix)?;
    for (index, item) in path.iter().enumerate() {
        if index > 0 {
            formatter.write_str(" -> ")?;
        }
        write!(formatter, "{item}")?;
    }
    Ok(())
}
