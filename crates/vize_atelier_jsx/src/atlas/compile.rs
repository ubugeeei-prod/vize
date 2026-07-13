//! Typed Atlas product for production JSX/TSX compilation.

#[path = "compile/codegen.rs"]
mod codegen;

use vize_atlas::{
    Compilation, CompilationInputError, PlanningContext, Product, ProductId, Provider,
    ProviderContext, ProviderError, SourceId, SourceInput, SourceInputId,
};
use vize_carton::{FxHashMap, String, cstr};
use vize_rendu::{RenderCapabilities, RenderCapabilitiesInput};

use crate::{JsxCompileConfig, JsxDiagnostic, JsxLang, ScopedStyle};

use super::{JsxRenderModuleProduct, is_jsx_source};

/// Complete output-affecting request for one JSX/TSX source.
#[derive(Debug, Clone, Default)]
pub struct JsxCompileRequest {
    pub lang: Option<JsxLang>,
    pub config: JsxCompileConfig,
}

impl JsxCompileRequest {
    pub fn new(lang: JsxLang, config: JsxCompileConfig) -> Self {
        Self {
            lang: Some(lang),
            config,
        }
    }
}

/// Source-aware JSX compile settings for persistent and batch compilations.
#[derive(Debug, Clone, Default)]
pub struct JsxCompileSettings {
    default: JsxCompileRequest,
    sources: FxHashMap<SourceId, JsxCompileRequest>,
}

impl JsxCompileSettings {
    pub fn new(default: JsxCompileRequest) -> Self {
        Self {
            default,
            sources: FxHashMap::default(),
        }
    }

    pub fn insert(&mut self, source: SourceId, request: JsxCompileRequest) {
        self.sources.insert(source, request);
    }

    pub fn get(&self, source: SourceId) -> &JsxCompileRequest {
        self.sources.get(&source).unwrap_or(&self.default)
    }

    /// Install all source overrides without invalidating unrelated JSX files.
    pub fn install(&self, compilation: &mut Compilation) -> Result<(), CompilationInputError> {
        for (source, request) in &self.sources {
            compilation.set_source_input::<JsxCompileSettingsInput>(*source, request.clone())?;
        }
        Ok(())
    }
}

/// Typed Atlas input carrying the full JSX compile request.
pub struct JsxCompileSettingsInput;

impl SourceInput for JsxCompileSettingsInput {
    type Value = JsxCompileRequest;
    const NAME: &'static str = "jsx.compile-settings";
}

/// Host-ready compiled module, diagnostics, source map, and scoped styles.
#[derive(Debug, Clone)]
pub struct JsxCompileArtifact {
    pub code: String,
    pub map: Option<String>,
    pub diagnostics: Vec<JsxDiagnostic>,
    pub scoped_styles: Vec<ScopedStyle>,
}

impl JsxCompileArtifact {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }
}

/// Complete JSX/TSX compiled module product.
pub struct JsxCompileProduct;

impl Product for JsxCompileProduct {
    type Value = JsxCompileArtifact;
    const NAME: &'static str = "jsx.compiled-module";
}

/// Production provider over the owned, component-boundary-preserving Rendu module.
pub struct JsxCompileProvider;

impl Provider for JsxCompileProvider {
    type Product = JsxCompileProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<JsxCompileSettingsInput>()]
    }

    fn supports(&self, context: &PlanningContext<'_>) -> bool {
        is_jsx_source(context.source().name())
    }

    fn dependencies(&self, context: &PlanningContext<'_>) -> Vec<ProductId> {
        let request = planning_request(context);
        let backends = backend_selection(&request.config, context.source().text());
        let mut dependencies = vec![ProductId::of::<JsxRenderModuleProduct>()];
        if backends.dom {
            dependencies.push(ProductId::of::<vize_atelier_dom::DomOutputProduct>());
        }
        if backends.ssr {
            dependencies.push(ProductId::of::<vize_atelier_ssr::SsrOutputProduct>());
        }
        if backends.vapor {
            dependencies.push(ProductId::of::<vize_atelier_vapor::VaporOutputProduct>());
        }
        dependencies
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<JsxCompileArtifact, ProviderError> {
        let render = context.get::<JsxRenderModuleProduct>()?;
        let request = context
            .source_input::<JsxCompileSettingsInput>()
            .cloned()
            .unwrap_or_default();
        let backends = backend_selection(&request.config, context.source().text());
        let dom = backends
            .dom
            .then(|| context.get::<vize_atelier_dom::DomOutputProduct>())
            .transpose()?;
        let ssr = backends
            .ssr
            .then(|| context.get::<vize_atelier_ssr::SsrOutputProduct>())
            .transpose()?;
        let vapor = backends
            .vapor
            .then(|| context.get::<vize_atelier_vapor::VaporOutputProduct>())
            .transpose()?;
        let output = codegen::compile_render_module(
            render.as_ref(),
            dom.as_deref(),
            ssr.as_deref(),
            vapor.as_deref(),
            context.source().name(),
            context.source().text(),
            &request.config,
        )
        .map_err(ProviderError::message)?;
        let code = output.module_code();
        let map = output.module_source_map(code.as_str(), context.source().name());
        let scoped_styles = output
            .components
            .iter()
            .filter_map(|component| component.scoped_style().cloned())
            .collect();
        Ok(JsxCompileArtifact {
            code,
            map,
            diagnostics: output.diagnostics,
            scoped_styles,
        })
    }
}

/// Execute the production typed product for one stateless host request.
pub fn compile_jsx_with_atlas(
    source: &str,
    filename: &str,
    lang: JsxLang,
    config: JsxCompileConfig,
) -> Result<JsxCompileArtifact, ProviderError> {
    let mut compilation = Compilation::new();
    super::register_atlas_providers(&mut compilation)
        .map_err(|error| ProviderError::message(cstr!("{error}")))?;
    let source_id = compilation
        .add_source(filename, source)
        .map_err(|error| ProviderError::message(cstr!("{error}")))?;
    let backends = backend_selection(&config, source);
    let mut settings = JsxCompileSettings::default();
    settings.insert(source_id, JsxCompileRequest::new(lang, config));
    settings
        .install(&mut compilation)
        .map_err(|error| ProviderError::message(cstr!("{error}")))?;
    compilation
        .set_input::<RenderCapabilitiesInput>(RenderCapabilities {
            dom: backends.dom,
            ssr: backends.ssr,
            vapor: backends.vapor,
            custom_renderer: false,
        })
        .map_err(|error| ProviderError::message(cstr!("{error}")))?;
    let outcome = compilation
        .query::<JsxCompileProduct>(source_id)
        .map_err(|error| ProviderError::message(cstr!("{error}")))?;
    Ok((*outcome.value()).clone())
}

#[derive(Debug, Clone, Copy)]
struct BackendSelection {
    dom: bool,
    ssr: bool,
    vapor: bool,
}

fn planning_request(context: &PlanningContext<'_>) -> JsxCompileRequest {
    context
        .source_input::<JsxCompileSettingsInput>()
        .cloned()
        .unwrap_or_default()
}

fn backend_selection(config: &JsxCompileConfig, source: &str) -> BackendSelection {
    if config.ssr {
        return BackendSelection {
            dom: false,
            ssr: true,
            vapor: false,
        };
    }
    let explicit_vdom = source.contains("\"use vue:vdom\"") || source.contains("'use vue:vdom'");
    let explicit_vapor = source.contains("\"use vue:vapor\"") || source.contains("'use vue:vapor'");
    BackendSelection {
        dom: config.default_mode == crate::JsxOutputMode::Vdom || explicit_vdom,
        ssr: false,
        vapor: config.default_mode == crate::JsxOutputMode::Vapor || explicit_vapor,
    }
}

#[cfg(test)]
#[path = "compile/tests.rs"]
mod tests;
