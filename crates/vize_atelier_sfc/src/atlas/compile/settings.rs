//! Narrow source-local inputs for each SFC compilation stage.

use vize_atlas::{Compilation, CompilationInputError, SourceId, SourceInput};
use vize_carton::{FxHashMap, String};
use vize_relief::TemplateSyntaxMode;

use crate::{SfcCompileOptions, SfcParseOptions};

/// Complete output-affecting request for one SFC source.
#[derive(Debug, Clone, Default)]
pub struct SfcCompileRequest {
    /// Complete public compiler options for this source.
    pub options: SfcCompileOptions,
    /// Parser compatibility mode for this source's template.
    pub template_syntax: TemplateSyntaxMode,
    /// Derive template/style scoped flags from the parsed descriptor before compilation.
    pub infer_scoped_from_descriptor: bool,
}

impl SfcCompileRequest {
    /// Create a request without descriptor-derived option normalization.
    pub fn new(options: SfcCompileOptions, template_syntax: TemplateSyntaxMode) -> Self {
        Self {
            options,
            template_syntax,
            infer_scoped_from_descriptor: false,
        }
    }

    /// Derive both template and style scoped flags from the cached descriptor.
    pub fn with_inferred_scoped_from_descriptor(mut self) -> Self {
        self.infer_scoped_from_descriptor = true;
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

/// Install one complete request as narrow, independently invalidated inputs.
pub fn install_sfc_compile_request(
    compilation: &mut Compilation,
    source: SourceId,
    request: SfcCompileRequest,
) -> Result<(), CompilationInputError> {
    let parse = request.options.parse.clone();
    if compilation
        .source_input::<SfcParseSettingsInput>(source)
        .is_none_or(|current| !parse_options_eq(current, &parse))
    {
        compilation.set_source_input::<SfcParseSettingsInput>(source, parse)?;
    }

    let vapor_script = compilation
        .source(source)
        .and_then(|source| crate::parse::scan_sfc_structure(source.text()))
        .is_some_and(|structure| structure.vapor_script);
    let frontend = SfcTemplateFrontendRequest::from_compile_request(&request, vapor_script);
    if compilation.source_input::<SfcTemplateFrontendSettingsInput>(source) != Some(&frontend) {
        compilation.set_source_input::<SfcTemplateFrontendSettingsInput>(source, frontend)?;
    }

    let render = SfcRenderRequest::from(&request);
    if compilation.source_input::<SfcRenderSettingsInput>(source) != Some(&render) {
        compilation.set_source_input::<SfcRenderSettingsInput>(source, render)?;
    }

    compilation.set_source_input::<SfcCompileSettingsInput>(source, request)?;
    Ok(())
}

fn parse_options_eq(left: &SfcParseOptions, right: &SfcParseOptions) -> bool {
    left.filename == right.filename
        && left.source_map == right.source_map
        && left.pad == right.pad
        && left.ignore_empty == right.ignore_empty
        && match (&left.template_parse_options, &right.template_parse_options) {
            (None, None) => true,
            (Some(left), Some(right)) => parser_options_eq(left, right),
            _ => false,
        }
}

#[allow(clippy::too_many_lines)]
fn parser_options_eq(
    left: &vize_relief::ParserOptions,
    right: &vize_relief::ParserOptions,
) -> bool {
    left.mode == right.mode
        && left.whitespace == right.whitespace
        && left.delimiters == right.delimiters
        && std::ptr::fn_addr_eq(left.is_pre_tag, right.is_pre_tag)
        && optional_fn_eq(left.is_native_tag, right.is_native_tag)
        && optional_fn_eq(left.is_custom_element, right.is_custom_element)
        && left.custom_renderer == right.custom_renderer
        && std::ptr::fn_addr_eq(left.is_void_tag, right.is_void_tag)
        && std::ptr::fn_addr_eq(left.get_namespace, right.get_namespace)
        && optional_handler_eq(left.on_error, right.on_error)
        && optional_handler_eq(left.on_warn, right.on_warn)
        && left.comments == right.comments
        && left.experimental_in_tag_comments == right.experimental_in_tag_comments
        && left.dialect == right.dialect
}

fn optional_fn_eq(left: Option<fn(&str) -> bool>, right: Option<fn(&str) -> bool>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => std::ptr::fn_addr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn optional_handler_eq(
    left: Option<fn(vize_relief::CompilerError)>,
    right: Option<fn(vize_relief::CompilerError)>,
) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => std::ptr::fn_addr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}
