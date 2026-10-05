//! Follow native declarations through the emitted bare-prop semantic edge.

use vize_canon::virtual_ts::VizeSemanticLinkKind;
use vize_canon::{CorsaBridge, LspLocation};

use crate::ide::corsa_support::CanonicalVirtualDocument;

pub(super) async fn declarations(
    bridge: &CorsaBridge,
    document: &CanonicalVirtualDocument,
    locations: Vec<LspLocation>,
) -> Option<Vec<LspLocation>> {
    let mut declarations = Vec::new();
    for location in locations {
        let target = (location.uri == document.request_uri.as_str())
            .then(|| {
                let start = crate::ide::position_to_offset(
                    &document.virtual_result.code,
                    location.range.start.line,
                    location.range.start.character,
                )?;
                let end = crate::ide::position_to_offset(
                    &document.virtual_result.code,
                    location.range.end.line,
                    location.range.end.character,
                )?;
                document
                    .virtual_result
                    .semantic_links
                    .iter()
                    .find(|link| {
                        link.kind == VizeSemanticLinkKind::VueTemplatePropBinding
                            && link.source_range == (start..end)
                    })
                    .map(|link| link.target_range.start)
            })
            .flatten();
        if let Some(target) = target {
            let (line, character) =
                crate::ide::offset_to_position(&document.virtual_result.code, target);
            declarations.extend(
                bridge
                    .definition(&document.request_uri, line, character)
                    .await
                    .ok()?,
            );
        } else {
            declarations.push(location);
        }
    }
    Some(declarations)
}
