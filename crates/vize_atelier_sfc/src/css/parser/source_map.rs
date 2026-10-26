//! Serialize only the existing CSS printer's actual input-rule origins.

use oxc_sourcemap::SourceMapBuilder;
use parcel_sourcemap::SourceMap;
use vize_carton::String;

pub(super) fn serialize(map: &SourceMap, filename: &str, css: &str) -> Option<String> {
    let mappings = map.get_mappings();
    if mappings.is_empty() || map.get_sources().len() != 1 {
        return None;
    }
    let mut builder = SourceMapBuilder::default();
    // Parcel's source registration normalizes paths relative to its project
    // root. This API owns one explicit input, so retain its exact public name
    // and bytes rather than presenting that internal relative spelling.
    let source = builder.set_source_and_content(filename, css);
    builder.set_file(filename);
    let names: Vec<_> = map
        .get_names()
        .iter()
        .map(|name| builder.add_name(name))
        .collect();
    for mapping in mappings {
        let (line, column, source, name) = match mapping.original {
            Some(origin) if origin.source == 0 => (
                origin.original_line,
                origin.original_column,
                Some(source),
                match origin.name {
                    Some(name) => Some(*names.get(name as usize)?),
                    None => None,
                },
            ),
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
    Some(builder.into_sourcemap().to_json_string().into())
}
