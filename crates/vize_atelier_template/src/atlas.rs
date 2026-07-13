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

/// Register only the raw-template frontend's product providers and compile recipe.
pub fn register_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    compilation.register_provider(RawTemplateReliefProvider)?;
    compilation.register_provider(RawTemplateCroquisProvider)?;
    compilation.register_provider(RawTemplateTransformedReliefProvider)?;
    compilation.register_provider(RawTemplateRenduProvider)?;
    compilation.register_provider(RawTemplateFlowProvider)?;
    compilation.register_provider(TemplateCompileProvider)?;
    Ok(())
}
