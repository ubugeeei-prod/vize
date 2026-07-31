use super::{EffectBuildContext, collect_program_effect_edges, context_for_program};
use crate::effect_graph::EffectGraph;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// One parsed SFC script block and its declared language.
#[derive(Debug, Clone, Copy)]
pub struct EffectGraphScript<'a> {
    /// Raw contents of one `<script>` block.
    pub source: &'a str,
    /// Declared SFC language (`js`, `jsx`, `ts`, or `tsx`).
    pub lang: Option<&'a str>,
}

impl<'a> EffectGraphScript<'a> {
    /// Create a script-block input for effect graph construction.
    pub const fn new(source: &'a str, lang: Option<&'a str>) -> Self {
        Self { source, lang }
    }

    fn virtual_path(self) -> &'static str {
        match self.lang.map(str::trim) {
            Some(lang) if lang.eq_ignore_ascii_case("jsx") => "script.jsx",
            Some(lang) if lang.eq_ignore_ascii_case("tsx") => "script.tsx",
            Some(lang) if lang.eq_ignore_ascii_case("ts") => "script.ts",
            _ => "script.js",
        }
    }
}

/// Build one effect graph across the normal and setup scopes of an SFC.
///
/// Normal-script bindings remain visible to setup, while scoped node IDs keep
/// same-named declarations in the two blocks distinct.
pub fn build_effect_graph_from_sfc_scripts(
    script: Option<EffectGraphScript<'_>>,
    setup: Option<EffectGraphScript<'_>>,
) -> EffectGraph {
    let mut graph = EffectGraph::default();
    let mut script_context = EffectBuildContext::default();

    if let Some(block) = script {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(block.virtual_path()).unwrap_or_default();
        let parsed = Parser::new(&allocator, block.source, source_type).parse();
        if !parsed.panicked && parsed.diagnostics.is_empty() {
            script_context = context_for_program(&parsed.program, None, "script");
            collect_program_effect_edges(&parsed.program, &script_context, &mut graph);
        }
    }

    if let Some(block) = setup {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(block.virtual_path()).unwrap_or_default();
        let parsed = Parser::new(&allocator, block.source, source_type).parse();
        if !parsed.panicked && parsed.diagnostics.is_empty() {
            let inherited = script.map(|_| &script_context);
            let setup_context = context_for_program(&parsed.program, inherited, "setup");
            collect_program_effect_edges(&parsed.program, &setup_context, &mut graph);
        }
    }

    graph
}
