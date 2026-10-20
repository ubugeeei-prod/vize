//! Resolve a config file's formatter options into native formatter options.

use vize_l0::config::FormatterProfile;

pub(super) fn format_options_from_config(profile: &FormatterProfile) -> vize_glyph::FormatOptions {
    vize_glyph::FormatOptions::from_profile(profile).unwrap_or_else(|error| {
        tracing::warn!("Invalid formatter configuration: {error}");
        vize_glyph::FormatOptions::default()
    })
}
