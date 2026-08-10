/// SFC block type used when mapping diagnostics back to authored source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfcBlockType {
    Template,
    Script,
    ScriptSetup,
    Style,
}

impl SfcBlockType {
    /// The SFC block name as it appears in a `.vue` file / `@vue/compiler-sfc`
    /// descriptor (`scriptSetup`, `script`, `template`, `style`).
    pub fn block_name(self) -> &'static str {
        match self {
            Self::Template => "template",
            Self::Script => "script",
            Self::ScriptSetup => "scriptSetup",
            Self::Style => "style",
        }
    }
}

/// Best-effort fallback byte offset for SFC diagnostics that ship without a
/// `loc`.
///
/// Returns the start of the most relevant block (`<script setup>`, then
/// `<script>`, then `<template>`) so the diagnostic lands somewhere clickable
/// instead of at file offset 0. This helper is intentionally available without
/// Canon's native checker feature because structural editor diagnostics use the
/// same source-location contract.
pub fn sfc_block_fallback_offset(
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> Option<(usize, SfcBlockType)> {
    if let Some(setup) = descriptor.script_setup.as_ref() {
        return Some((setup.loc.start, SfcBlockType::ScriptSetup));
    }
    if let Some(script) = descriptor.script.as_ref() {
        return Some((script.loc.start, SfcBlockType::Script));
    }
    if let Some(template) = descriptor.template.as_ref() {
        return Some((template.loc.start, SfcBlockType::Template));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{SfcBlockType, sfc_block_fallback_offset};

    fn parse(source: &str) -> vize_atelier_sfc::SfcDescriptor<'_> {
        vize_atelier_sfc::parse_sfc(source, vize_atelier_sfc::SfcParseOptions::default()).unwrap()
    }

    #[test]
    fn fallback_prefers_script_setup_without_native_checker() {
        let source = "<template />\n<script>export default {}</script>\n<script setup>const value = 1</script>";
        let descriptor = parse(source);

        assert_eq!(
            sfc_block_fallback_offset(&descriptor),
            Some((
                source.find("const value").unwrap(),
                SfcBlockType::ScriptSetup
            ))
        );
    }

    #[test]
    fn fallback_uses_script_then_template_and_handles_empty_sfc() {
        let script_source = "<template />\n<script>export default {}</script>";
        let script = parse(script_source);
        assert_eq!(
            sfc_block_fallback_offset(&script),
            Some((
                script_source.find("export default").unwrap(),
                SfcBlockType::Script
            ))
        );

        let template_source = "<template><div /></template>";
        let template = parse(template_source);
        assert_eq!(
            sfc_block_fallback_offset(&template),
            Some((
                template_source.find("<div").unwrap(),
                SfcBlockType::Template
            ))
        );

        assert_eq!(sfc_block_fallback_offset(&parse("")), None);
    }
}
