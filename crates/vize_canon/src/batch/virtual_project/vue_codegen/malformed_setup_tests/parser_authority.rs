use super::GeneratedVueFile;

pub(super) fn packet(output: &GeneratedVueFile) -> serde_json::Value {
    let imports = crate::batch::import_rewriter::ImportRewriter::new()
        .collect_all_specifiers(&output.code, oxc_span::SourceType::ts().with_module(true));
    serde_json::json!({
        "wholeCode":output.code.as_str(),
        "wholeMappings":vize_carton::cstr!("{:#?}",output.mappings),
        "wholeSemanticLinks":vize_carton::cstr!("{:#?}",output.semantic_links),
        "wholePropDefaultKeyLinks":vize_carton::cstr!("{:#?}",output.prop_default_key_links),
        "wholeLocalDiagnostics":vize_carton::cstr!("{:#?}",output.diagnostics),
        "typedRouterImport":output.typed_router_import,
        "wholeActualImportRewriterSpecifiers":imports,
    })
}
