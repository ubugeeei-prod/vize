//! Per-file compile settings and template-syntax mapping for the build command.

use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::config::VueVersion;

use crate::commands::build::ScriptExtension;

/// Compile a single `.vue` file with profiling information.
#[derive(Clone, Copy)]
pub(super) struct CompileFileSettings {
    pub(super) ssr: bool,
    pub(super) vapor: bool,
    pub(super) custom_renderer: bool,
    pub(super) template_syntax: TemplateSyntaxMode,
    /// Vue dialect from `vue.version`; defaults to [`VueVersion::V3`] and is
    /// threaded into each file's compile options.
    pub(super) dialect: VueVersion,
    pub(super) script_ext: ScriptExtension,
    pub(super) record_profile_totals: bool,
}

impl CompileFileSettings {
    /// Packs every compile option that can change stats output into a tiny cache key.
    ///
    /// `record_profile_totals` is intentionally excluded: enabling profiling changes
    /// accounting side effects, not parse/compile output. Script extension is included
    /// because preserving TypeScript can change generated code size. The dialect is
    /// included because it can change parse/compile output on legacy-capable builds.
    pub(super) fn cache_bits(self) -> u16 {
        u16::from(self.ssr)
            | (u16::from(self.vapor) << 1)
            | (u16::from(self.custom_renderer) << 2)
            | (u16::from(template_syntax_bits(self.template_syntax)) << 3)
            | match self.script_ext {
                ScriptExtension::Preserve => 1 << 5,
                ScriptExtension::Downcompile => 0,
            }
            | (u16::from(dialect_bits(self.dialect)) << 6)
    }
}

fn dialect_bits(dialect: VueVersion) -> u8 {
    match dialect {
        VueVersion::V3 => 0,
        VueVersion::V2_7 => 1,
        VueVersion::V2 => 2,
        VueVersion::V1 => 3,
        VueVersion::V0_11 => 4,
        VueVersion::V0_10 => 5,
    }
}

fn template_syntax_bits(template_syntax: TemplateSyntaxMode) -> u8 {
    match template_syntax {
        TemplateSyntaxMode::Standard => 0,
        TemplateSyntaxMode::Strict => 1,
        TemplateSyntaxMode::Quirks => 2,
        _ => 3,
    }
}

pub(super) fn template_syntax_mode(template_syntax: Option<&str>) -> TemplateSyntaxMode {
    match template_syntax {
        Some("strict") => TemplateSyntaxMode::Strict,
        Some("quirks") => TemplateSyntaxMode::Quirks,
        Some("standard") | None => TemplateSyntaxMode::Standard,
        Some(_) => TemplateSyntaxMode::Standard,
    }
}
