//! Source store failures.

use std::{error::Error, fmt};

use crate::{SourceId, SourceRange};

/// Failures while adding or updating sources.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SourceError {
    SourceNotFound(SourceId),
    SourceIdExhausted,
    RevisionOverflow(SourceId),
    InvalidEmbeddedRange {
        parent: SourceId,
        range: SourceRange,
        parent_len: usize,
    },
    RangeNotCharBoundary {
        parent: SourceId,
        range: SourceRange,
    },
    StaleParent(SourceId),
    NotEmbedded(SourceId),
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNotFound(source) => write!(formatter, "source {source} does not exist"),
            Self::SourceIdExhausted => formatter.write_str("source identity space is exhausted"),
            Self::RevisionOverflow(source) => write!(formatter, "revision overflow for {source}"),
            Self::InvalidEmbeddedRange {
                parent,
                range,
                parent_len,
            } => write!(
                formatter,
                "embedded range {}..{} is outside {parent} (length {parent_len})",
                range.start, range.end
            ),
            Self::RangeNotCharBoundary { parent, range } => write!(
                formatter,
                "embedded range {}..{} is not on UTF-8 boundaries in {parent}",
                range.start, range.end
            ),
            Self::StaleParent(parent) => {
                write!(
                    formatter,
                    "cannot derive a source from stale parent {parent}"
                )
            }
            Self::NotEmbedded(source) => write!(formatter, "source {source} is not embedded"),
        }
    }
}

impl Error for SourceError {}
