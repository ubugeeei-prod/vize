//! Narrow source-local inputs for each SFC compilation stage.

#[path = "settings/equality.rs"]
mod equality;

use vize_atlas::{
    Compilation, CompilationInputError, SourceId, SourceInput, SourceKind, SourceKindInput,
};
use vize_carton::{FxHashMap, String};
use vize_relief::TemplateSyntaxMode;
use vize_rendu::{RenderEmitSettings, RenderEmitSettingsInput};

use crate::atlas::croquis::SfcInferredCroquisSettingsInput;
use crate::{SFC_SOURCE_KIND, SfcCompileOptions, SfcParseOptions};
use crate::{SfcCroquisMode, SfcCroquisRequest};

use self::equality::parse_options_eq;

/// Complete output-affecting request for one SFC source.
#[derive(Debug, Clone, Default)]
pub struct SfcCompileRequest {
    /// Complete public compiler options for this source.
    pub options: SfcCompileOptions,
    /// Parser compatibility mode for this source's template.
    pub template_syntax: TemplateSyntaxMode,
    /// Derive template/style scoped flags from the parsed descriptor before compilation.
    pub infer_scoped_from_descriptor: bool,
    /// Frontend-neutral JavaScript packaging settings consumed by the selected backend.
    pub render_emit: RenderEmitSettings,
}

impl SfcCompileRequest {
    /// Create a request without descriptor-derived option normalization.
    pub fn new(options: SfcCompileOptions, template_syntax: TemplateSyntaxMode) -> Self {
        Self {
            options,
            template_syntax,
            infer_scoped_from_descriptor: false,
            render_emit: RenderEmitSettings::default(),
        }
    }

    /// Derive both template and style scoped flags from the cached descriptor.
    pub fn with_inferred_scoped_from_descriptor(mut self) -> Self {
        self.infer_scoped_from_descriptor = true;
        self
    }

    /// Override the runtime names used by every Rendu backend and standalone assembly.
    pub fn with_runtime_names(
        mut self,
        runtime_module_name: impl Into<String>,
        runtime_global_name: impl Into<String>,
    ) -> Self {
        self.render_emit.runtime_module_name = runtime_module_name.into();
        self.render_emit.runtime_global_name = runtime_global_name.into();
        self
    }
}

/// Source-aware settings for a multi-file compilation.
#[derive(Debug, Clone, Default)]
pub struct SfcCompileSettings {
    default: SfcCompileRequest,
    sources: FxHashMap<SourceId, SfcCompileRequest>,
}

impl SfcCompileSettings {
    pub fn new(default: SfcCompileRequest) -> Self {
        Self {
            default,
            sources: FxHashMap::default(),
        }
    }

    pub fn set_default(&mut self, request: SfcCompileRequest) {
        self.default = request;
    }

    pub fn insert(&mut self, source: SourceId, request: SfcCompileRequest) {
        self.sources.insert(source, request);
    }

    pub fn get(&self, source: SourceId) -> &SfcCompileRequest {
        self.sources.get(&source).unwrap_or(&self.default)
    }

    /// Install stage-specific inputs without invalidating unrelated products.
    pub fn install(&self, compilation: &mut Compilation) -> Result<(), CompilationInputError> {
        for (source, request) in &self.sources {
            install_sfc_compile_request(compilation, *source, request.clone())?;
        }
        Ok(())
    }
}

/// Complete request consumed only by final SFC module assembly.
pub struct SfcCompileSettingsInput;

impl SourceInput for SfcCompileSettingsInput {
    type Value = SfcCompileRequest;
    const NAME: &'static str = "sfc.compile-settings";
}

/// Container parser options, independent from script/style/backend settings.
pub struct SfcParseSettingsInput;

impl SourceInput for SfcParseSettingsInput {
    type Value = SfcParseOptions;
    const NAME: &'static str = "sfc.parse-settings";
}

/// Exact settings consumed by Relief parsing and template transforms.
#[derive(Debug, Clone, Default)]
pub struct SfcTemplateFrontendRequest {
    pub filename: String,
    pub template_syntax: TemplateSyntaxMode,
    pub ssr: bool,
    pub vapor: bool,
    pub template_is_ts: bool,
    pub script_is_ts: bool,
    pub custom_renderer: bool,
    pub dialect: vize_carton::config::VueVersion,
    pub comments: bool,
    pub experimental_in_tag_comments: bool,
    pub experimental_patterned_template: bool,
    source_vapor: bool,
}

impl SfcTemplateFrontendRequest {
    fn from_compile_request(request: &SfcCompileRequest, vapor_script: bool) -> Self {
        let compiler = request.options.template.compiler_options.as_ref();
        Self {
            filename: request.options.parse.filename.clone(),
            template_syntax: request.template_syntax,
            ssr: request.options.template.ssr,
            vapor: request.options.vapor,
            template_is_ts: request.options.template.is_ts,
            script_is_ts: request.options.script.is_ts,
            custom_renderer: request.options.template.custom_renderer,
            dialect: request.options.template.dialect,
            comments: compiler.is_some_and(|options| options.comments),
            experimental_in_tag_comments: compiler
                .is_some_and(|options| options.experimental_in_tag_comments),
            experimental_patterned_template: compiler
                .is_some_and(|options| options.experimental_patterned_template),
            source_vapor: vapor_script,
        }
    }

    fn installed_vapor_mode(&self) -> bool {
        !self.ssr && (self.vapor || self.source_vapor)
    }
}

impl PartialEq for SfcTemplateFrontendRequest {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.template_syntax == other.template_syntax
            && self.installed_vapor_mode() == other.installed_vapor_mode()
            && self.template_is_ts == other.template_is_ts
            && self.script_is_ts == other.script_is_ts
            && self.custom_renderer == other.custom_renderer
            && self.dialect == other.dialect
            && self.comments == other.comments
            && self.experimental_in_tag_comments == other.experimental_in_tag_comments
            && self.experimental_patterned_template == other.experimental_patterned_template
    }
}

impl Eq for SfcTemplateFrontendRequest {}

pub struct SfcTemplateFrontendSettingsInput;

impl SourceInput for SfcTemplateFrontendSettingsInput {
    type Value = SfcTemplateFrontendRequest;
    const NAME: &'static str = "sfc.template-frontend-settings";
}

/// Backend routing settings, independent from frontend parsing and transforms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SfcRenderRequest {
    pub ssr: bool,
    pub vapor: bool,
}

impl From<&SfcCompileRequest> for SfcRenderRequest {
    fn from(request: &SfcCompileRequest) -> Self {
        Self {
            ssr: request.options.template.ssr,
            vapor: request.options.vapor,
        }
    }
}

pub struct SfcRenderSettingsInput;

impl SourceInput for SfcRenderSettingsInput {
    type Value = SfcRenderRequest;
    const NAME: &'static str = "sfc.render-settings";
}

/// Scope identity consumed only while producing frontend-neutral render HIR.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct SfcRenderScopeRequest {
    pub scope_id: String,
}

pub(crate) struct SfcRenderScopeSettingsInput;

impl SourceInput for SfcRenderScopeSettingsInput {
    type Value = SfcRenderScopeRequest;
    const NAME: &'static str = "sfc.render-scope-settings";
}

/// Install one complete request as narrow, independently invalidated inputs.
pub fn install_sfc_compile_request(
    compilation: &mut Compilation,
    source: SourceId,
    request: SfcCompileRequest,
) -> Result<(), CompilationInputError> {
    let source_kind = SourceKind::new(SFC_SOURCE_KIND);
    if compilation.source_input::<SourceKindInput>(source) != Some(&source_kind) {
        compilation.set_source_input::<SourceKindInput>(source, source_kind)?;
    }
    let parse = request.options.parse.clone();
    if compilation
        .source_input::<SfcParseSettingsInput>(source)
        .is_none_or(|current| !parse_options_eq(current, &parse))
    {
        compilation.set_source_input::<SfcParseSettingsInput>(source, parse)?;
    }

    let structure = compilation
        .source(source)
        .and_then(|source| crate::parse::scan_sfc_structure(source.text()))
        .unwrap_or_default();
    let vapor_script = structure.vapor_script;
    let mode = if request.options.template.dialect.is_legacy() {
        SfcCroquisMode::LegacyVue2
    } else if structure.has_normal_script {
        SfcCroquisMode::OptionsApi
    } else {
        SfcCroquisMode::Full
    };
    let resolved_filename = request
        .options
        .script
        .id
        .as_ref()
        .filter(|filename| !filename.is_empty())
        .cloned()
        .or_else(|| {
            (!request.options.parse.filename.is_empty())
                .then(|| request.options.parse.filename.clone())
        })
        .or_else(|| {
            compilation
                .source(source)
                .map(|source| source.name())
                .filter(|filename| !filename.is_empty())
                .map(Into::into)
        });
    let inferred = SfcCroquisRequest {
        mode,
        resolved_filename,
        ..Default::default()
    };
    if compilation.source_input::<SfcInferredCroquisSettingsInput>(source) != Some(&inferred) {
        compilation.set_source_input::<SfcInferredCroquisSettingsInput>(source, inferred)?;
    }
    let frontend = SfcTemplateFrontendRequest::from_compile_request(&request, vapor_script);
    if compilation.source_input::<SfcTemplateFrontendSettingsInput>(source) != Some(&frontend) {
        compilation.set_source_input::<SfcTemplateFrontendSettingsInput>(source, frontend)?;
    }

    let render = SfcRenderRequest::from(&request);
    if compilation.source_input::<SfcRenderSettingsInput>(source) != Some(&render) {
        compilation.set_source_input::<SfcRenderSettingsInput>(source, render)?;
    }

    let scope_filename = if request.options.parse.filename.as_str().is_empty() {
        request
            .options
            .script
            .id
            .as_deref()
            .filter(|filename| !filename.is_empty())
            .unwrap_or_else(|| {
                compilation
                    .source(source)
                    .map_or("anonymous.vue", |source| source.name())
            })
    } else {
        request.options.parse.filename.as_str()
    };
    let render_scope = SfcRenderScopeRequest {
        scope_id: request
            .options
            .scope_id
            .clone()
            .unwrap_or_else(|| crate::compile::generate_scope_id(scope_filename)),
    };
    if compilation.source_input::<SfcRenderScopeSettingsInput>(source) != Some(&render_scope) {
        compilation.set_source_input::<SfcRenderScopeSettingsInput>(source, render_scope)?;
    }

    if compilation.source_input::<RenderEmitSettingsInput>(source) != Some(&request.render_emit) {
        compilation
            .set_source_input::<RenderEmitSettingsInput>(source, request.render_emit.clone())?;
    }

    compilation.set_source_input::<SfcCompileSettingsInput>(source, request)?;
    Ok(())
}
