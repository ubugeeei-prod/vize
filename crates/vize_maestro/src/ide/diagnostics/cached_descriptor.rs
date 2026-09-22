//! The diagnostics request path reads the resident parse (P5-6b).
//!
//! A rejected buffer is the same memo as a successful one: the parser
//! diagnostic is built from the stored error, and a second request for that
//! revision does not parse again.
#![allow(clippy::disallowed_methods)]

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use vize_resident::{DescriptorParseError, ParsedSfc, SharedDescriptor};

use super::{DiagnosticService, sources};
use crate::server::ServerState;

impl DiagnosticService {
    /// The document's resident descriptor, or the one SFC parser diagnostic
    /// the collect path publishes before skipping dependent diagnostics.
    #[allow(clippy::result_large_err)]
    pub(super) fn descriptor_for_collect(
        state: &ServerState,
        uri: &Url,
        content: &str,
    ) -> Result<SharedDescriptor, Diagnostic> {
        match state.sfc_parsed(uri, content) {
            ParsedSfc::Descriptor(descriptor) => Ok(descriptor),
            ParsedSfc::Failed(error) => Err(parser_diagnostic(&error)),
        }
    }
}

fn parser_diagnostic(error: &DescriptorParseError) -> Diagnostic {
    let range = error.loc.map_or_else(Range::default, |loc| Range {
        start: Position {
            line: loc.start_line.saturating_sub(1) as u32,
            character: loc.start_column.saturating_sub(1) as u32,
        },
        end: Position {
            line: loc.end_line.saturating_sub(1) as u32,
            character: loc.end_column.saturating_sub(1) as u32,
        },
    });
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(sources::SFC_PARSER.to_string()),
        message: error.message.to_string(),
        ..Default::default()
    }
}
