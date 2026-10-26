//! The shared Canon projection for JavaScript language-service adapters.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "the N-API boundary exchanges JavaScript strings"
)]

use std::path::Path;

use napi::bindgen_prelude::*;
use napi_derive::napi;
use vize_canon::{
    ContentMapperProjectConfiguration, ContentMapperTransformOptions,
    generate_vue_content_mapper_transform_with_options,
};

/// Explicit host-selected options. Source text always comes from the caller.
#[napi(object)]
pub struct VueLanguageCodeOptions {
    pub filename: String,
    /// An explicit tsconfig; omitted means no configuration file reads.
    pub config_file_name: Option<String>,
    pub options_api: Option<bool>,
    #[napi(js_name = "checkJavaScript")]
    pub check_javascript: Option<bool>,
    pub preserve_unused_diagnostics: Option<bool>,
}

/// Serialize the existing UTF-8 Content Mapper protocol, without another emitter.
#[napi(js_name = "generateVueLanguageCode")]
pub fn generate_vue_language_code(
    source: String,
    options: VueLanguageCodeOptions,
) -> Result<String> {
    let mut transform_options = match options.config_file_name.as_deref() {
        Some(config) => {
            ContentMapperProjectConfiguration::load(Path::new(config))
                .map_err(|error| Error::from_reason(error.to_string()))?
                .transform_options
        }
        None => ContentMapperTransformOptions::default(),
    };
    if let Some(enabled) = options.options_api {
        transform_options = transform_options.with_options_api(enabled);
    }
    if let Some(enabled) = options.check_javascript {
        transform_options = transform_options.with_check_javascript(enabled);
    }
    if let Some(enabled) = options.preserve_unused_diagnostics {
        transform_options = transform_options.with_preserve_unused_diagnostics(enabled);
    }
    let transform = generate_vue_content_mapper_transform_with_options(
        Path::new(&options.filename),
        &source,
        transform_options,
    )
    .map_err(|error| Error::from_reason(error.to_string()))?;
    serde_json::to_string(&transform).map_err(|error| Error::from_reason(error.to_string()))
}
