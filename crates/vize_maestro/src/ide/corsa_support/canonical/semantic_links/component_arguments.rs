//! Authored property-key roles retained from resolved native endpoints.

use tower_lsp::lsp_types::Location;
use vize_canon::{LspLocation, LspPosition, LspRange};

use super::{CanonicalVirtualDocument, ComponentPropNavigationMatches};
use crate::ide::IdeContext;

impl ComponentPropNavigationMatches {
    /// Retain the authored geometry of each definition-verified property key.
    /// The same Vue shorthand token also projects a value binding; only the
    /// producer-owned navigation endpoint establishes its public-key role.
    pub(crate) fn authored_arguments(
        &self,
        ctx: &IdeContext<'_>,
        document: &CanonicalVirtualDocument,
    ) -> Option<Vec<Location>> {
        self.positions
            .iter()
            .map(|position| {
                let (_, result) = super::virtual_result(document, &position.request_uri)?;
                let offset = crate::ide::position_to_offset(
                    &result.code,
                    position.line,
                    position.character,
                )?;
                let link = result.semantic_links.iter().find(|link| {
                    link.kind
                        == vize_canon::virtual_ts::VizeSemanticLinkKind::VueComponentPropNavigation
                        && link.target_range.start == offset
                })?;
                let (end_line, end_character) =
                    crate::ide::offset_to_position(&result.code, link.target_range.end);
                super::super::map_canonical_corsa_location(
                    ctx,
                    document,
                    &LspLocation {
                        uri: position.request_uri.to_string(),
                        range: LspRange {
                            start: LspPosition {
                                line: position.line,
                                character: position.character,
                            },
                            end: LspPosition {
                                line: end_line,
                                character: end_character,
                            },
                        },
                    },
                )
            })
            .collect()
    }
}
