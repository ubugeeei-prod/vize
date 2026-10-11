use super::GeneratedVueFile;

pub(super) fn packet(output: &GeneratedVueFile) -> serde_json::Value {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(
        &allocator,
        &output.code,
        oxc_span::SourceType::ts().with_module(true),
    )
    .parse();
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
        "wholeOxcParser":{
            "panicked":parsed.panicked,
            "programBodyLength":parsed.program.body.len(),
            "diagnostics":vize_carton::cstr!("{:#?}",parsed.diagnostics),
        },
    })
}
