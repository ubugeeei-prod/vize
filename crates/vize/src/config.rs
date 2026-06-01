//! Shared config helpers for the CLI.

use std::{fs, path::Path};

pub use vize_carton::config::*;

pub const VIZE_CONFIG_SCHEMA: &str =
    include_str!("../../../npm/vize/schemas/vize.config.schema.json");

/// Write the JSON Schema to `node_modules/.vize/vize.config.schema.json`.
pub fn write_schema(dir: Option<&Path>) {
    let base = dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let schema_dir = base.join("node_modules/.vize");
    if fs::create_dir_all(&schema_dir).is_ok() {
        let schema_path = schema_dir.join("vize.config.schema.json");
        if schema_needs_write(&schema_path) {
            let _ = fs::write(&schema_path, VIZE_CONFIG_SCHEMA);
        }
    }
}

fn schema_needs_write(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) if metadata.len() == VIZE_CONFIG_SCHEMA.len() as u64 => {
            fs::read_to_string(path).map_or(true, |current| current.as_str() != VIZE_CONFIG_SCHEMA)
        }
        Ok(_) | Err(_) => true,
    }
}

#[cfg(feature = "glyph")]
#[must_use]
pub fn to_glyph_format_options(config: &FormatterConfig) -> vize_glyph::FormatOptions {
    vize_glyph::FormatOptions {
        print_width: config.print_width,
        tab_width: config.tab_width,
        use_tabs: config.use_tabs,
        semi: config.semi,
        single_quote: config.single_quote,
        jsx_single_quote: config.jsx_single_quote,
        trailing_comma: match config.trailing_comma {
            TrailingComma::None => vize_glyph::TrailingComma::None,
            TrailingComma::Es5 => vize_glyph::TrailingComma::Es5,
            TrailingComma::All => vize_glyph::TrailingComma::All,
        },
        bracket_spacing: config.bracket_spacing,
        bracket_same_line: config.bracket_same_line,
        arrow_parens: match config.arrow_parens {
            ArrowParens::Always => vize_glyph::ArrowParens::Always,
            ArrowParens::Avoid => vize_glyph::ArrowParens::Avoid,
        },
        end_of_line: match config.end_of_line {
            EndOfLine::Lf => vize_glyph::EndOfLine::Lf,
            EndOfLine::Crlf => vize_glyph::EndOfLine::Crlf,
            EndOfLine::Cr => vize_glyph::EndOfLine::Cr,
            EndOfLine::Auto => vize_glyph::EndOfLine::Auto,
        },
        quote_props: match config.quote_props {
            QuoteProps::AsNeeded => vize_glyph::QuoteProps::AsNeeded,
            QuoteProps::Consistent => vize_glyph::QuoteProps::Consistent,
            QuoteProps::Preserve => vize_glyph::QuoteProps::Preserve,
        },
        single_attribute_per_line: config.single_attribute_per_line,
        vue_indent_script_and_style: config.vue_indent_script_and_style,
        sort_attributes: config.sort_attributes,
        attribute_sort_order: match config.attribute_sort_order {
            AttributeSortOrder::Alphabetical => vize_glyph::AttributeSortOrder::Alphabetical,
            AttributeSortOrder::AsWritten => vize_glyph::AttributeSortOrder::AsWritten,
        },
        merge_bind_and_non_bind_attrs: config.merge_bind_and_non_bind_attrs,
        max_attributes_per_line: config.max_attributes_per_line,
        attribute_groups: config.attribute_groups.clone(),
        normalize_directive_shorthands: config.normalize_directive_shorthands,
        sort_blocks: config.sort_blocks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_needs_write_when_missing() {
        let project = tempfile::tempdir().unwrap();

        assert!(schema_needs_write(
            &project.path().join("vize.config.schema.json")
        ));
    }

    #[test]
    fn schema_needs_write_when_stale() {
        let project = tempfile::tempdir().unwrap();
        let schema_path = project.path().join("vize.config.schema.json");
        fs::write(&schema_path, "{}").unwrap();

        assert!(schema_needs_write(&schema_path));
    }

    #[test]
    fn schema_write_is_skipped_when_current() {
        let project = tempfile::tempdir().unwrap();
        let schema_path = project.path().join("vize.config.schema.json");
        fs::write(&schema_path, VIZE_CONFIG_SCHEMA).unwrap();

        assert!(!schema_needs_write(&schema_path));
    }

    #[test]
    fn write_schema_refreshes_stale_schema() {
        let project = tempfile::tempdir().unwrap();
        let schema_dir = project.path().join("node_modules/.vize");
        fs::create_dir_all(&schema_dir).unwrap();
        let schema_path = schema_dir.join("vize.config.schema.json");
        fs::write(&schema_path, "{}").unwrap();

        write_schema(Some(project.path()));

        assert_eq!(fs::read_to_string(schema_path).unwrap(), VIZE_CONFIG_SCHEMA);
    }
}
