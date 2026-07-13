//! Atlas providers for independently supplied raw Vue templates.

mod output;
mod providers;
mod settings;
mod source_map;

#[cfg(test)]
mod tests;

pub use output::{
    TemplateCompileArtifact, TemplateCompileProduct, TemplateCompileProvider, TemplateOutputMapping,
};
pub use providers::{
    RawTemplateCroquisProvider, RawTemplateFlowProvider, RawTemplateReliefProvider,
    RawTemplateRenduProvider, RawTemplateTransformedReliefProvider,
};
pub use settings::{
    RAW_TEMPLATE_SUFFIX, TemplateCompileRequest, TemplateCompileSettingsInput, TemplateParseMode,
    TemplateParseModeInput, TemplateRenderTarget, install_template_compile_request,
    install_template_parse_mode,
};

use vize_atlas::{Compilation, RegisterProviderError};

/// Register the raw-template frontend and frontend-neutral target backends.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(RawTemplateReliefProvider)?;
    compilation.register_provider(RawTemplateCroquisProvider)?;
    compilation.register_provider(RawTemplateTransformedReliefProvider)?;
    compilation.register_provider(RawTemplateRenduProvider)?;
    compilation.register_provider(RawTemplateFlowProvider)?;
    vize_atelier_dom::register_atlas_provider(compilation)?;
    vize_atelier_ssr::register_atlas_provider(compilation)?;
    vize_atelier_vapor::register_atlas_provider(compilation)?;
    compilation.register_provider(TemplateCompileProvider)?;
    if !compilation.has_provider::<vize_croquis::CroquisSemanticProduct>() {
        vize_croquis::register_semantic_projection(compilation)?;
    }
    Ok(())
}
