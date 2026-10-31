//! Project formatting settings with CLI overrides.

use super::{FmtArgs, FmtEndOfLine, FormatOptions};
use crate::config;
use clap::ValueEnum;
use vize_glyph::VueVersion;

/// Named formatter defaults selectable from the CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FmtPreset {
    /// The Vize defaults
    Vize,
    /// Prettier's Vue layout, as listed in the formatter guide
    Prettier,
    /// Dense application layout that preserves authored order
    Compact,
    /// Readable documentation examples with one attribute per line
    Documentation,
}

impl From<FmtPreset> for vize_glyph::FormatPreset {
    fn from(preset: FmtPreset) -> Self {
        match preset {
            FmtPreset::Vize => Self::Vize,
            FmtPreset::Prettier => Self::Prettier,
            FmtPreset::Compact => Self::Compact,
            FmtPreset::Documentation => Self::Documentation,
        }
    }
}

/// Attribute order modes selectable from the CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub(in crate::commands) enum FmtAttributeSortOrder {
    /// Sort alphabetically within each priority group
    Alphabetical,
    /// Preserve the authored order within each priority group
    AsWritten,
}

impl From<FmtAttributeSortOrder> for vize_glyph::AttributeSortOrder {
    fn from(value: FmtAttributeSortOrder) -> Self {
        match value {
            FmtAttributeSortOrder::Alphabetical => Self::Alphabetical,
            FmtAttributeSortOrder::AsWritten => Self::AsWritten,
        }
    }
}

impl From<FmtEndOfLine> for vize_glyph::EndOfLine {
    fn from(value: FmtEndOfLine) -> Self {
        match value {
            FmtEndOfLine::Lf => Self::Lf,
            FmtEndOfLine::Crlf => Self::Crlf,
            FmtEndOfLine::Cr => Self::Cr,
            FmtEndOfLine::Auto => Self::Auto,
        }
    }
}

/// Build format options: config file as base, CLI flags override.
#[inline]
pub(super) fn build_format_options(
    args: &FmtArgs,
    snapshot: &config::LoadedFormatterSnapshot,
) -> (
    FormatOptions,
    VueVersion,
    Option<vize_glyph::ImportSortOptions>,
) {
    let sort_imports = vize_glyph::resolve_sort_imports(snapshot.sort_imports.as_ref())
        .unwrap_or_else(|error| {
            eprintln!("Invalid formatter configuration: {error}");
            std::process::exit(2);
        });
    let mut opts =
        FormatOptions::from_profile(&snapshot.formatter_profile).unwrap_or_else(|error| {
            eprintln!("Invalid formatter configuration: {error}");
            std::process::exit(2);
        });
    if let Some(preset) = args.preset {
        opts.apply_preset(preset.into());
    }

    // CLI flags override config values
    if let Some(v) = args.print_width {
        opts.print_width = v;
    }
    if let Some(v) = args.tab_width {
        opts.tab_width = v;
    }
    if let Some(v) = args.use_tabs {
        opts.use_tabs = v;
    }
    if args.no_semi {
        opts.semi = false;
    }
    if let Some(v) = args.single_quote {
        opts.single_quote = v;
    }
    if let Some(v) = args.jsx_single_quote {
        opts.jsx_single_quote = v;
    }
    if let Some(v) = args.quote_props {
        opts.quote_props = v.into();
    }
    if let Some(v) = args.trailing_comma {
        opts.trailing_comma = v.into();
    }
    if let Some(v) = args.arrow_parens {
        opts.arrow_parens = v.into();
    }
    if let Some(v) = args.bracket_spacing {
        opts.bracket_spacing = v;
    }
    if let Some(v) = args.bracket_same_line {
        opts.bracket_same_line = v;
    }
    if let Some(v) = args.end_of_line {
        opts.end_of_line = v.into();
    }
    if let Some(v) = args.sort_attributes {
        opts.sort_attributes = v;
    }
    if let Some(v) = args.attribute_sort_order {
        opts.attribute_sort_order = v.into();
    }
    if let Some(v) = args.single_attribute_per_line {
        opts.single_attribute_per_line = v;
    }
    if let Some(v) = args.vue_indent_script_and_style {
        opts.vue_indent_script_and_style = v;
    }
    if let Some(v) = args.max_attributes_per_line {
        opts.max_attributes_per_line = Some(v);
    }
    if let Some(v) = args.normalize_directive_shorthands {
        opts.normalize_directive_shorthands = v;
    }
    if let Some(v) = args.sort_blocks {
        opts.sort_blocks = v;
    }
    if let Some(v) = args.self_close_void_elements {
        opts.self_close_void_elements = v;
    }
    if let Some(v) = args.normalize_css_quotes {
        opts.normalize_css_quotes = v;
    }
    if let Some(v) = args.normalize_inline_styles {
        opts.normalize_inline_styles = v;
    }
    if let Some(v) = args.normalize_v_for_parens {
        opts.normalize_v_for_parens = v;
    }
    if let Some(v) = args.reindent_preprocessor_styles {
        opts.reindent_preprocessor_styles = v;
    }
    opts.skip_script_stabilization = !args.write;
    (
        opts,
        snapshot.loaded.features.vue_version.unwrap_or_default(),
        sort_imports,
    )
}
