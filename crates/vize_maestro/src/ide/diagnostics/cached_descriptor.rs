//! The diagnostics request path reads the resident parse (P5-6b).
//!
//! A rejected buffer is the same memo as a successful one: the parser
//! diagnostic is built from the stored error, and a second request for that
//! revision does not parse again.
#![expect(
    clippy::disallowed_methods,
    reason = "tower-lsp lsp_types take std String/HashMap values, built with to_string/format!"
)]

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use vize_incremental::{DescriptorParseError, ParsedSfc, SharedDescriptor};

use super::{DiagnosticService, LineIndex, sources};
use crate::server::ServerState;

impl DiagnosticService {
    /// The document's resident descriptor, or the one SFC parser diagnostic
    /// the collect path publishes before skipping dependent diagnostics.
    #[expect(
        clippy::result_large_err,
        reason = "the parser diagnostic is returned by value to the single collect caller"
    )]
    pub(super) fn descriptor_for_collect(
        state: &ServerState,
        uri: &Url,
        content: &str,
    ) -> Result<SharedDescriptor, Diagnostic> {
        match state.sfc_parsed(uri, content) {
            ParsedSfc::Descriptor(descriptor) => Ok(descriptor),
            ParsedSfc::Failed(error) => Err(parser_diagnostic(&error, content)),
        }
    }
}

fn parser_diagnostic(error: &DescriptorParseError, content: &str) -> Diagnostic {
    let range = error.loc.map_or_else(Range::default, |loc| {
        let index = LineIndex::new(content);
        let (start_line, start_character) = index.line_col(loc.start);
        let (end_line, end_character) = index.line_col(loc.end);
        Range {
            start: Position {
                line: start_line,
                character: start_character,
            },
            end: Position {
                line: end_line,
                character: end_character,
            },
        }
    });
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        source: Some(sources::SFC_PARSER.to_string()),
        message: error.message.to_string(),
        ..Default::default()
    }
}
