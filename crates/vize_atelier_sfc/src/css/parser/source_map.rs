//! Serialize only the existing CSS printer's actual input-rule origins.

use oxc_sourcemap::SourceMapBuilder;
use parcel_sourcemap::SourceMap;
use vize_carton::String;
use vize_croquis::sfc::__internal::CssVBindEdit;

mod positions;
#[cfg(test)]
mod tests;
use positions::Positions;

#[derive(Clone, Copy)]
pub(in crate::css) struct CssSourceMapInput<'a> {
    pub source: &'a str,
    pub edits: &'a [CssVBindEdit],
}

fn copied_origin(byte: usize, edits: &[CssVBindEdit]) -> Option<usize> {
    let end = edits.partition_point(|edit| edit.generated.start <= byte);
    let Some(edit) = end.checked_sub(1).and_then(|at| edits.get(at)) else {
        return Some(byte);
    };
    (byte >= edit.generated.end).then(|| edit.source.end + byte - edit.generated.end)
}

pub(super) fn serialize(
    map: &SourceMap,
    filename: &str,
    css: &str,
    input: CssSourceMapInput<'_>,
) -> Option<String> {
    let mappings = map.get_mappings();
    if mappings.is_empty() || map.get_sources().len() != 1 {
        return None;
    }
    let mut builder = SourceMapBuilder::default();
    // Parcel's source registration normalizes paths relative to its project
    // root. This API owns one explicit input, so retain its exact public name
    // and bytes rather than presenting that internal relative spelling.
    let source = builder.set_source_and_content(filename, input.source);
    builder.set_file(filename);
    let names: Vec<_> = map
        .get_names()
        .iter()
        .map(|name| builder.add_name(name))
        .collect();
    // Only rewritten inputs need coordinate indices. They contain sparse
    // Unicode corrections, avoiding repeated prefix walks on minified lines.
    let positions =
        (!input.edits.is_empty()).then(|| (Positions::new(css), Positions::new(input.source)));
    let mut owned = false;
    for mapping in mappings {
        let (line, column, source, name) = match mapping.original {
            Some(origin) if origin.source == 0 => {
                let location = match &positions {
                    Some((generated, original)) => generated
                        .byte(origin.original_line, origin.original_column)
                        .and_then(|at| copied_origin(at, input.edits))
                        .and_then(|at| original.location(at)),
                    None => Some((origin.original_line, origin.original_column)),
                };
                let name = match origin.name {
                    Some(name) => Some(*names.get(name as usize)?),
                    None => None,
                };
                match location {
                    Some((line, column)) => {
                        owned = true;
                        (line, column, Some(source), name)
                    }
                    None => (0, 0, None, None),
                }
            }
            Some(_) => return None,
            None => (0, 0, None, None),
        };
        builder.add_token(
            mapping.generated_line,
            mapping.generated_column,
            line,
            column,
            source,
            name,
        );
    }
    owned.then(|| builder.into_sourcemap().to_json_string().into())
}
