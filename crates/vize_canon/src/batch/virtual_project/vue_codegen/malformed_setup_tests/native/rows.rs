//! Map every native diagnostic through the retained original projection.

use super::GeneratedVueFile;
use crate::{corsa_client::LspDiagnostic, file_uri::file_uri_to_path};
use std::path::Path;
use vize_carton::String;

pub(super) type Row = (
    std::string::String,
    u32,
    u32,
    Option<u32>,
    Option<i32>,
    std::string::String,
);

pub(super) fn mapped_rows(
    root: &Path,
    inputs: &[(&str, std::string::String, GeneratedVueFile)],
    raw: &[(String, Vec<LspDiagnostic>)],
) -> Vec<Row> {
    let mut rows = Vec::new();
    for (uri, diagnostics) in raw {
        let path = file_uri_to_path(uri).unwrap();
        let (name, source, output) = inputs
            .iter()
            .find_map(|(name, source, output)| {
                (path == root.join(vize_carton::cstr!("{name}.ts").as_str()))
                    .then_some((*name, source, output))
            })
            .unwrap();
        let source_map = crate::batch::source_map::SfcSourceMap::new_with_semantic_links(
            output.mappings.clone(),
            Vec::new(),
            output.semantic_links.clone(),
        );
        for diagnostic in diagnostics {
            let start = &diagnostic.range.start;
            let Some(offset) = crate::batch::source_map::line_col_to_offset(
                &output.code,
                start.line,
                start.character,
            ) else {
                continue;
            };
            let Some((original, _, _)) = source_map.get_original_position(offset) else {
                continue;
            };
            let (line, column) =
                crate::batch::source_map::offset_to_line_col(source, original).unwrap();
            let code = diagnostic.code.as_ref().and_then(|code| {
                code.as_u64()
                    .and_then(|code| u32::try_from(code).ok())
                    .or_else(|| code.as_str().and_then(|code| code.parse().ok()))
            });
            rows.push((
                name.into(),
                line,
                column,
                code,
                diagnostic.severity,
                std::string::String::from(diagnostic.message.as_str()),
            ));
        }
    }
    rows.sort();
    rows
}
