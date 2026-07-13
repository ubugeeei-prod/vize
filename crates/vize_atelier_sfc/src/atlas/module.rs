//! Parse-once SFC script frontend and neutral module projection.

#[path = "module/parse.rs"]
mod parse;

use std::cell::Cell;

use vize_atlas::{
    ObservationKind, PlanningContext, Product, ProductId, Provider, ProviderContext, ProviderError,
    SourceInputId, SourceKindInput, SourceRange,
};
use vize_carton::source_anchor::SourceAnchor;
use vize_croquis::{Croquis, script_parser::ScriptParseResult};
use vize_module::{ModuleDocument, ModuleSyntaxProduct};

use crate::compile_script::{NormalScriptCompilerFacts, PreanalyzedScriptSetup};
use crate::types::SfcMacroArtifact;

use super::croquis::{
    SfcCroquisSettingsInput, SfcInferredCroquisSettingsInput, croquis_request_for_provider,
};
use super::script_generator::SfcScriptGeneratorFacts;
use super::{SfcDescriptorProduct, is_sfc_context, source_structure};
use parse::{parse_plain, parse_setup};

/// Owned script facts derived while each SFC script block's OXC program is live.
///
/// The snapshot retains both Vue-specific script analysis and the neutral
/// module projection. No allocator-bound OXC node escapes into Atlas.
#[derive(Debug, Clone, Default)]
pub struct SfcScriptSyntaxSnapshot {
    modules: ModuleDocument,
    plain: Option<PlainScriptAnalysis>,
    setup: Option<ScriptParseResult>,
    setup_compiler: Option<PreanalyzedScriptSetup>,
    generator: SfcScriptGeneratorFacts,
}

impl SfcScriptSyntaxSnapshot {
    /// Neutral JavaScript/TypeScript facts and CFG for the authored blocks.
    pub fn module(&self) -> &ModuleDocument {
        &self.modules
    }

    /// Owned facts consumed by downstream script generators without reparsing.
    pub fn generator_facts(&self) -> &SfcScriptGeneratorFacts {
        &self.generator
    }

    /// Validate Vue-specific script-setup semantics from the live Program
    /// projection retained by this snapshot.
    pub fn validate_script_setup_semantics(&self, sfc_source: &str) -> Result<(), crate::SfcError> {
        self.setup_compiler
            .as_ref()
            .map_or(Ok(()), |setup| setup.validate_semantics(sfc_source))
    }

    pub(crate) fn setup_compiler(&self) -> Option<&PreanalyzedScriptSetup> {
        self.setup_compiler.as_ref()
    }

    pub(crate) fn normal_compiler(&self) -> Option<&NormalScriptCompilerFacts> {
        self.plain.as_ref().map(|plain| &plain.compiler)
    }

    pub(crate) fn macro_artifacts(&self) -> Vec<SfcMacroArtifact> {
        self.plain
            .iter()
            .flat_map(|plain| plain.compiler.macro_artifacts())
            .chain(
                self.setup_compiler
                    .iter()
                    .flat_map(|setup| setup.macro_artifacts()),
            )
            .cloned()
            .collect()
    }

    pub(super) fn croquis(&self, merge_scripts: bool) -> Croquis {
        match (&self.plain, &self.setup) {
            (Some(plain), Some(setup)) if merge_scripts => {
                let plain = plain.semantics.clone().into_croquis();
                let mut summary = setup.clone().into_croquis();
                let setup_offset = self.modules.modules.first().map_or(0, |module| {
                    u32::try_from(module.source.len())
                        .unwrap_or(u32::MAX)
                        .saturating_add(1)
                });
                summary.shift_script_offsets(setup_offset);
                summary.merge_plain_script(plain);
                summary
            }
            (_, Some(setup)) => setup.clone().into_croquis(),
            (Some(plain), None) => plain.semantics.clone().into_croquis(),
            (None, None) => Croquis::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct PlainScriptAnalysis {
    semantics: ScriptParseResult,
    compiler: NormalScriptCompilerFacts,
}

thread_local! {
    static AUTHORED_SCRIPT_PARSE_INVOCATIONS: Cell<u64> = const { Cell::new(0) };
}

#[doc(hidden)]
pub fn authored_script_parse_invocations() -> u64 {
    AUTHORED_SCRIPT_PARSE_INVOCATIONS.get()
}

#[doc(hidden)]
pub fn reset_authored_script_parse_invocations() {
    AUTHORED_SCRIPT_PARSE_INVOCATIONS.set(0);
}

/// SFC-owned script syntax product shared by Module, Croquis, Canon, and compile.
pub struct SfcScriptSyntaxProduct;

impl Product for SfcScriptSyntaxProduct {
    type Value = SfcScriptSyntaxSnapshot;

    const NAME: &'static str = "sfc.script-syntax";
}

/// Parse every authored script block once and materialize all owned projections.
pub struct SfcScriptSyntaxProvider;

impl Provider for SfcScriptSyntaxProvider {
    type Product = SfcScriptSyntaxProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![
            SourceInputId::of::<SfcCroquisSettingsInput>(),
            SourceInputId::of::<SfcInferredCroquisSettingsInput>(),
            SourceInputId::of::<SourceKindInput>(),
        ]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_context(context) && source_structure(context).has_script
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcDescriptorProduct>()]
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<SfcScriptSyntaxSnapshot, ProviderError> {
        let descriptor = context.get::<SfcDescriptorProduct>()?;
        let Some(descriptor) = descriptor.descriptor() else {
            return Ok(SfcScriptSyntaxSnapshot::default());
        };
        let source = context.source();
        let root_anchor = SourceAnchor::new(source.id().get(), source.revision().get());
        let mut snapshot = SfcScriptSyntaxSnapshot::default();
        let semantic_mode = croquis_request_for_provider(context).mode;

        if let Some(script) = descriptor.script.as_ref() {
            let (module, analysis, generator) = parse_plain(
                source.name(),
                script.content.as_ref(),
                script.lang.as_deref(),
                script.loc.start,
                script.loc.end,
                root_anchor,
                semantic_mode,
            )?;
            snapshot.modules.modules.push(module);
            snapshot.generator.merge(generator);
            snapshot.plain = Some(analysis);
        }
        if let Some(script) = descriptor.script_setup.as_ref() {
            let generic = script.attrs.get("generic").map(|value| value.as_ref());
            let (module, analysis, compiler, generator) = parse_setup(
                source.name(),
                script.content.as_ref(),
                script.lang.as_deref(),
                generic,
                script.loc.start,
                script.loc.end,
                root_anchor,
                snapshot.plain.as_ref(),
            )?;
            snapshot.modules.modules.push(module);
            snapshot.generator.merge(generator);
            snapshot.setup = Some(analysis);
            snapshot.setup_compiler = Some(compiler);
        }
        Ok(snapshot)
    }
}

/// Project the parse-once SFC snapshot into the production-neutral module product.
pub struct SfcModuleSyntaxProvider;

impl Provider for SfcModuleSyntaxProvider {
    type Product = ModuleSyntaxProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<SourceKindInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_sfc_context(context) && source_structure(context).has_script
    }

    fn dependencies(&self, _context: &PlanningContext<'_>) -> Vec<ProductId> {
        vec![ProductId::of::<SfcScriptSyntaxProduct>()]
    }

    fn provide(&self, context: &mut ProviderContext<'_>) -> Result<ModuleDocument, ProviderError> {
        let syntax = context.get::<SfcScriptSyntaxProduct>()?;
        for module in &syntax.module().modules {
            for diagnostic in &module.diagnostics {
                context.observe(
                    ObservationKind::Diagnostic,
                    "module.parse.error",
                    diagnostic.message.as_ref(),
                    Some(SourceRange::new(
                        diagnostic.span.start as usize,
                        diagnostic.span.end as usize,
                    )),
                );
            }
        }
        Ok(syntax.module().clone())
    }
}
