use vize_s0::String;

use super::source_map::SourceMapBuilder;

/// Build a minimal one-source v3 map anchored at the generated module start.
///
/// P3-9 migrates DOM/Vapor/SSR to structured span-carrying emitters in slices.
/// This helper gives non-DOM emitters an opt-in Source Map v3 contract before
/// their individual tokens move onto [`SourceMapBuilder`] segments.
pub fn build_single_anchor_source_map(
    generated_code: &str,
    filename: &str,
    source_content: &str,
) -> String {
    let mut builder = SourceMapBuilder::new();
    builder.add_raw(0, 0);
    builder.finish(generated_code, filename, source_content)
}
