use vize_atlas::{
    Compilation, CompilationInputError, SourceId, SourceInput, SourceInputInvalidationReport,
};
use vize_carton::{FxHashMap, FxHashSet, String};

/// Descriptor analysis compatibility selected for one SFC source.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SfcCroquisMode {
    /// Composition API and script-setup-first Vue 3 analysis.
    #[default]
    Full,
    /// Include Vue 3 Options API template bindings.
    OptionsApi,
    /// Include Vue 2.7 / Nuxt 2 Options API bindings and template globals.
    LegacyVue2,
    /// Script-only declaration analysis without template parsing or traversal.
    Declaration,
}

/// Imported-prop resolution policy for one SFC semantic product.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SfcResolvedPropsPolicy {
    /// Complete bindings before template traversal so semantic facts are clean.
    #[default]
    BeforeTemplate,
    /// Preserve legacy Canon bytes, including stale undefined-ref guard emits.
    ///
    /// This exists only for the zero-drift BatchTypeChecker migration. It still
    /// performs exactly one template traversal and one imported-type resolution.
    PreserveCanonAfterTemplate,
}

/// Complete semantic request attached to one SFC source.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SfcCroquisRequest {
    pub mode: SfcCroquisMode,
    pub resolved_filename: Option<String>,
    pub resolved_props_policy: SfcResolvedPropsPolicy,
}

/// Source-aware Croquis compatibility settings for a multi-file compilation.
#[derive(Debug, Clone, Default)]
pub struct SfcCroquisSettings {
    default: SfcCroquisMode,
    sources: FxHashMap<SourceId, SfcCroquisMode>,
    resolved_filenames: FxHashMap<SourceId, String>,
    resolved_policies: FxHashMap<SourceId, SfcResolvedPropsPolicy>,
}

impl SfcCroquisSettings {
    /// Create settings with one fallback analysis mode.
    pub fn new(default: SfcCroquisMode) -> Self {
        Self {
            default,
            sources: FxHashMap::default(),
            resolved_filenames: FxHashMap::default(),
            resolved_policies: FxHashMap::default(),
        }
    }

    /// Replace the fallback analysis mode.
    pub fn set_default(&mut self, mode: SfcCroquisMode) {
        self.default = mode;
    }

    /// Install or replace one source-specific analysis mode.
    pub fn insert(&mut self, source: SourceId, mode: SfcCroquisMode) {
        self.sources.insert(source, mode);
    }

    /// Resolve imported and heritage props relative to this source filename.
    ///
    /// Consumers that do not need filesystem-backed type resolution can omit
    /// this entry and retain the path-independent analysis used by lint/IDE
    /// graphs. Canon installs it for production project type checking.
    pub fn insert_resolved_filename(&mut self, source: SourceId, filename: impl Into<String>) {
        self.resolved_filenames.insert(source, filename.into());
        self.resolved_policies.remove(&source);
    }

    /// Resolve one filename with an explicit compatibility policy.
    pub fn insert_resolved_filename_with_policy(
        &mut self,
        source: SourceId,
        filename: impl Into<String>,
        policy: SfcResolvedPropsPolicy,
    ) {
        self.resolved_filenames.insert(source, filename.into());
        self.resolved_policies.insert(source, policy);
    }

    /// Resolve a source-specific mode or the fallback mode.
    pub fn get(&self, source: SourceId) -> SfcCroquisMode {
        self.sources.get(&source).copied().unwrap_or(self.default)
    }

    /// Return the source-aware filename used for imported type resolution.
    pub fn resolved_filename(&self, source: SourceId) -> Option<&str> {
        self.resolved_filenames
            .get(&source)
            .map(|filename| filename.as_str())
    }

    /// Imported-prop compatibility policy for one resolved source.
    pub fn resolved_props_policy(&self, source: SourceId) -> SfcResolvedPropsPolicy {
        self.resolved_policies
            .get(&source)
            .copied()
            .unwrap_or_default()
    }

    /// Install every explicit source request with source-local invalidation.
    pub fn install(
        self,
        compilation: &mut Compilation,
    ) -> Result<Vec<SourceInputInvalidationReport>, CompilationInputError> {
        let mut sources: FxHashSet<_> = self.sources.keys().copied().collect();
        sources.extend(self.resolved_filenames.keys().copied());
        sources.extend(self.resolved_policies.keys().copied());
        let mut reports = Vec::with_capacity(sources.len());
        for source in sources {
            reports.push(compilation.set_source_input::<SfcCroquisSettingsInput>(
                source,
                SfcCroquisRequest {
                    mode: self.get(source),
                    resolved_filename: self.resolved_filenames.get(&source).cloned(),
                    resolved_props_policy: self.resolved_props_policy(source),
                },
            )?);
        }
        Ok(reports)
    }
}

/// Typed Atlas input for source-aware SFC Croquis compatibility.
pub struct SfcCroquisSettingsInput;

impl SourceInput for SfcCroquisSettingsInput {
    type Value = SfcCroquisRequest;

    const NAME: &'static str = "sfc.croquis-settings";
}
