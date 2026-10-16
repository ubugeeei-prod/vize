//! Independent consumers must share the same core types across all six libraries.

pub fn facade_error(value: corsa_core::CorsaError) -> corsa::CorsaError {
    value
}
pub fn jsonrpc_error(value: corsa_core::CorsaError) -> corsa_jsonrpc::CorsaError {
    value
}
pub fn client_error(value: corsa_core::CorsaError) -> corsa_client::CorsaError {
    value
}
pub fn lsp_error(value: corsa_core::CorsaError) -> corsa_lsp::CorsaError {
    value
}
pub fn orchestrator_error(value: corsa_core::CorsaError) -> corsa_orchestrator::CorsaError {
    value
}
pub fn receipt(value: corsa_core::ProcessReceipt) -> corsa_lsp::process::ProcessReceipt {
    value
}

pub fn command(observer: corsa_core::ProcessObserver) -> corsa_lsp::LspSpawnConfig {
    let command = corsa::process::CorsaCommand::new("unexecuted").with_process_observer(observer);
    let mut api = corsa_client::ApiSpawnConfig::new("unexecuted");
    api.command = command;
    let mut lsp = corsa_lsp::LspSpawnConfig::new("unexecuted");
    lsp.command = api.command;
    lsp
}

pub fn local_config(
    value: corsa_orchestrator::ApiOrchestratorConfig,
) -> corsa::orchestrator::ApiOrchestratorConfig {
    value
}

// These APIs were added by the currently resolved client 1.14.2. Keeping them
// in a real consumer prevents an owned-graph import from downgrading them.
pub fn source_file(value: corsa_client::EncodedSourceFile) -> corsa::api::EncodedSourceFile {
    value
}

pub fn content_mapping(value: corsa_client::ContentMapping) -> corsa::api::ContentMapping {
    value
}

pub fn semantic_query<'a>(value: corsa_client::SemanticQuery<'a>) -> corsa::api::SemanticQuery<'a> {
    value
}

#[test]
fn current_client_content_mapping_and_semantics_work() {
    use corsa_client::{SpanMap, SpanMapFidelity, SpanMapSegment};
    let mapping = SpanMap::new([SpanMapSegment::verbatim(0, 12, 8, 20)]);
    let mapped = mapping.virtual_to_original_position(3);
    assert_eq!(mapped.position, 11);
    assert_eq!(mapped.fidelity, SpanMapFidelity::Exact);
    assert!(corsa_client::EncodedSourceFile::decode(&[]).is_err());
    assert_eq!(
        corsa::api::SEMANTIC_QUERY_VERSION,
        corsa_client::SEMANTIC_QUERY_VERSION
    );
    assert_eq!(corsa_client::SignatureKind::Call.as_wire_value(), 0);
    assert_eq!(corsa::api::NameMeaning::Namespace.as_wire_value(), 1920);
}

#[cfg(feature = "distributed")]
pub fn distributed_config(
    value: corsa_orchestrator::RaftConfig,
) -> corsa::orchestrator::RaftConfig {
    value
}

#[cfg(feature = "distributed")]
pub fn distributed_orchestrator(
    value: corsa_orchestrator::DistributedApiOrchestrator,
) -> corsa::orchestrator::DistributedApiOrchestrator {
    value
}

#[test]
fn facade_runtime_and_virtual_documents_work() -> corsa::Result<()> {
    use corsa::lsp::{VirtualChange, VirtualDocument};
    let mut document = VirtualDocument::untitled("/consumer.ts", "typescript", "const value = 1;")?;
    document.apply_changes(&[VirtualChange::replace("const value = 2;")])?;
    assert_eq!(document.text, "const value = 2;");
    assert_eq!(corsa::runtime::block_on(async { 21 * 2 }), 42);
    Ok(())
}
