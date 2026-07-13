use vize_atlas::{
    Compilation, CompilationInputError, PlanningContext, SourceId, SourceInput, SourceKind,
    SourceKindInput,
};
use vize_carton::String;
use vize_relief::{CodegenMode, ParserOptions, TemplateSyntaxMode, TransformOptions};
use vize_rendu::{RenderEmitSettings, RenderEmitSettingsInput, RenderOutputMode};

/// Explicit applicability suffix for raw Vue-template sources.
pub const RAW_TEMPLATE_SUFFIX: &str = ".vue-template";
/// Open Atlas source-kind value owned by the raw-template frontend.
pub const RAW_TEMPLATE_SOURCE_KIND: &str = "vue-template";

/// Container interpretation for an independently supplied template source.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum TemplateParseMode {
    /// A Vue template fragment, such as the content of an SFC template block.
    #[default]
    Fragment,
    /// A complete HTML document, including doctype and raw script/style tags.
    Document,
}

/// Typed per-source container interpretation for the raw-template frontend.
pub struct TemplateParseModeInput;

impl SourceInput for TemplateParseModeInput {
    type Value = TemplateParseMode;

    const NAME: &'static str = "template.parse-mode";
}

/// Backend selected for one raw template query.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum TemplateRenderTarget {
    #[default]
    Dom,
    Ssr,
    Vapor,
}

/// Complete source-scoped request for raw template compilation.
#[derive(Debug, Clone)]
pub struct TemplateCompileRequest {
    pub target: TemplateRenderTarget,
    pub template_syntax: TemplateSyntaxMode,
    pub parser: ParserOptions,
    pub transform: TransformOptions,
    pub mode: CodegenMode,
    pub source_map: bool,
    pub runtime_module_name: String,
    pub runtime_global_name: String,
}

impl Default for TemplateCompileRequest {
    fn default() -> Self {
        Self {
            target: TemplateRenderTarget::Dom,
            template_syntax: TemplateSyntaxMode::Standard,
            parser: ParserOptions::default(),
            transform: TransformOptions::default(),
            mode: CodegenMode::Function,
            source_map: false,
            runtime_module_name: "vue".into(),
            runtime_global_name: "Vue".into(),
        }
    }
}

impl TemplateCompileRequest {
    pub fn for_target(mut self, target: TemplateRenderTarget) -> Self {
        self.target = target;
        self.transform.ssr = matches!(target, TemplateRenderTarget::Ssr);
        self.transform.vapor = matches!(target, TemplateRenderTarget::Vapor);
        self
    }
}

/// Typed per-source raw-template compiler settings.
pub struct TemplateCompileSettingsInput;

impl SourceInput for TemplateCompileSettingsInput {
    type Value = TemplateCompileRequest;

    const NAME: &'static str = "template.compile-settings";
}

/// Install frontend and backend settings atomically before taking a snapshot.
pub fn install_template_compile_request(
    compilation: &mut Compilation,
    source: SourceId,
    request: TemplateCompileRequest,
) -> Result<(), CompilationInputError> {
    let source_kind = SourceKind::new(RAW_TEMPLATE_SOURCE_KIND);
    if compilation.source_input::<SourceKindInput>(source) != Some(&source_kind) {
        compilation.set_source_input::<SourceKindInput>(source, source_kind)?;
    }
    let emit = RenderEmitSettings {
        mode: match request.mode {
            CodegenMode::Function => RenderOutputMode::Function,
            CodegenMode::Module => RenderOutputMode::Module,
        },
        runtime_module_name: request.runtime_module_name.clone(),
        runtime_global_name: request.runtime_global_name.clone(),
    };
    compilation.set_source_input::<TemplateCompileSettingsInput>(source, request)?;
    compilation.set_source_input::<RenderEmitSettingsInput>(source, emit)?;
    Ok(())
}

/// Select fragment or full-document parsing for one raw-template source.
pub fn install_template_parse_mode(
    compilation: &mut Compilation,
    source: SourceId,
    mode: TemplateParseMode,
) -> Result<(), CompilationInputError> {
    compilation
        .set_source_input::<TemplateParseModeInput>(source, mode)
        .map(|_| ())
}

pub(super) fn is_raw_template_source(name: &str) -> bool {
    name.ends_with(RAW_TEMPLATE_SUFFIX)
}

pub(super) fn is_raw_template_context(context: &PlanningContext<'_>) -> bool {
    context.source_input::<SourceKindInput>().map_or_else(
        || is_raw_template_source(context.source().name()),
        |kind| kind.is(RAW_TEMPLATE_SOURCE_KIND),
    )
}
