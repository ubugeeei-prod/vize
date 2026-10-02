//! Project formatting settings with CLI overrides.

use super::{FmtArgs, FormatOptions};
use crate::config;
use vize_glyph::VueVersion;

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
    let mut opts = config::to_glyph_format_options(&snapshot.loaded.config.formatter);

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
    if let Some(v) = args.sort_attributes {
        opts.sort_attributes = v;
    }
    if let Some(v) = args.single_attribute_per_line {
        opts.single_attribute_per_line = v;
    }
    if let Some(v) = args.max_attributes_per_line {
        opts.max_attributes_per_line = Some(v);
    }
    if let Some(v) = args.normalize_directive_shorthands {
        opts.normalize_directive_shorthands = v;
    }
    opts.skip_script_stabilization = !args.write;
    (
        opts,
        snapshot.loaded.features.vue_version.unwrap_or_default(),
        sort_imports,
    )
}
