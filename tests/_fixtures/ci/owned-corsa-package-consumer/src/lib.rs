//! Compile-only consumers of the six actual packaged libraries.

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
    let command = corsa::process::CorsaCommand::new("unexecuted")
        .with_process_observer(observer);
    let mut api = corsa_client::ApiSpawnConfig::new("unexecuted");
    api.command = command;
    let mut lsp = corsa_lsp::LspSpawnConfig::new("unexecuted");
    lsp.command = api.command;
    lsp
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
