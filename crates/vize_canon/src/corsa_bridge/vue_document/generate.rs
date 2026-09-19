use super::*;

pub(super) fn generate_vue_document_with_options(
    source_path: &Path,
    content: &str,
    options: CorsaVueVirtualDocumentOptions,
    virtual_ts_options: &VirtualTsOptions,
    rewriter: &ImportRewriter,
    alias_context: Option<&super::super::vue_dependencies_alias::AliasContext>,
) -> Result<GeneratedVueDocument, CorsaBridgeError> {
    if let Some((path, generated)) =
        alias_context.and_then(|context| context.editor_vue_document(source_path))
    {
        return Ok(GeneratedVueDocument {
            source_path: source_path.to_path_buf(),
            virtual_uri: path_to_file_uri(&path),
            generated,
        });
    }
    let source_dir = source_path.parent().map(std::path::Path::to_path_buf);
    let alias_resolver = alias_context.zip(source_dir).map(|(context, dir)| {
        move |specifier: &str, mode| context.resolve_specifier_to_mirror_path(specifier, &dir, mode)
    });
    let generated = crate::batch::virtual_project::generate_vue_document_virtual_ts_with_options_and_alias_resolver(
        source_path,
        content,
        virtual_ts_options,
        rewriter,
        false,
        VueDocumentVirtualTsOptions {
            options_api: options.options_api,
            legacy_vue2: options.legacy_vue2,
            experimental_patterned_template: options.experimental_patterned_template,
            preserve_event_navigation: options.preserve_event_navigation,
            dialect: options.dialect,
            preserve_missing_vue_diagnostics: true,
        },
        alias_resolver
            .as_ref()
            .map(|resolver| resolver as crate::batch::import_rewriter_alias::AliasSpecifierResolver<'_>),
    )
    .map_err(|error| CorsaBridgeError::CommunicationError(cstr!("{error}")))?;
    let virtual_path = alias_context
        .and_then(|context| context.mirror_virtual_path(source_path))
        .unwrap_or_else(|| {
            source_path.with_file_name(cstr!(
                "{}{}",
                source_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default(),
                generated.virtual_suffix
            ))
        });
    let virtual_uri = path_to_file_uri(&virtual_path);

    Ok(GeneratedVueDocument {
        source_path: source_path.to_path_buf(),
        virtual_uri,
        generated,
    })
}
