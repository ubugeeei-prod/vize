//! Why the JS plugin host refused a plugin or a document (P4-16 spike).

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

use core::fmt;

/// Every host refusal, rendered as the exact message JS receives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostError {
    /// The SFC container could not be split.
    Split(String),
    /// A manifest demanded a fact group the host does not expose.
    UnknownFact {
        plugin: String,
        name: String,
        known: &'static [&'static str],
    },
    /// A manifest listed a node kind S2 does not have.
    UnknownKind { plugin: String, kind: String },
    /// Caching needs an explicit, unique list of plugin-owned ambient inputs.
    InvalidCacheInputs { plugin: String, detail: String },
    /// A plugin's `run` returned something other than a report array.
    BadReports { plugin: String, detail: String },
    /// A report named a node id outside the document.
    UnknownNode {
        plugin: String,
        rule: String,
        node: u32,
    },
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Split(detail) => write!(f, "cannot split the SFC: {detail}"),
            Self::UnknownFact {
                plugin,
                name,
                known,
            } => write!(
                f,
                "{plugin}: fact group `{name}` is not available to JS plugins (available: {})",
                known.join(", ")
            ),
            Self::UnknownKind { plugin, kind } => {
                write!(f, "{plugin}: `{kind}` is not an S2 node kind")
            }
            Self::InvalidCacheInputs { plugin, detail } => {
                write!(f, "{plugin}: invalid cacheInputs ({detail})")
            }
            Self::BadReports { plugin, detail } => {
                write!(
                    f,
                    "{plugin}: run() must return a JSON report array ({detail})"
                )
            }
            Self::UnknownNode { plugin, rule, node } => {
                write!(
                    f,
                    "{plugin}/{rule}: reported node {node}, which the document does not have"
                )
            }
        }
    }
}
