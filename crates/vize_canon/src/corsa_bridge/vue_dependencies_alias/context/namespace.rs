//! Stable private namespace for one native project configuration.

use std::path::Path;

pub(super) fn editor_namespace_identity(
    options: crate::corsa_bridge::vue_document::CorsaVueVirtualDocumentOptions,
    virtual_ts_options: &crate::virtual_ts::VirtualTsOptions,
    project_root: Option<&Path>,
    tsconfig_path: Option<&Path>,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut generation_options = std::hash::DefaultHasher::new();
    options.options_api.hash(&mut generation_options);
    options.legacy_vue2.hash(&mut generation_options);
    options.jsx_typecheck.hash(&mut generation_options);
    options
        .experimental_patterned_template
        .hash(&mut generation_options);
    options.dialect.hash(&mut generation_options);
    options
        .preserve_event_navigation
        .hash(&mut generation_options);
    for global in &virtual_ts_options.template_globals {
        global.name.hash(&mut generation_options);
        global.type_annotation.hash(&mut generation_options);
        global.default_value.hash(&mut generation_options);
    }
    virtual_ts_options.css_modules.hash(&mut generation_options);
    virtual_ts_options
        .auto_import_stubs
        .hash(&mut generation_options);
    virtual_ts_options
        .external_template_bindings
        .hash(&mut generation_options);
    virtual_ts_options
        .reference_paths
        .hash(&mut generation_options);
    project_root.hash(&mut generation_options);
    tsconfig_path.hash(&mut generation_options);
    generation_options.finish()
}
