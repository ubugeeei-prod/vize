use std::fs;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};

use super::DefinitionService;
use crate::{ide::IdeContext, server::ServerState};

mod component_props;
mod imported_symbols;
mod package_specifiers;
mod template_bindings;

fn scalar_location(response: GotoDefinitionResponse) -> Location {
    match response {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(mut locations) => {
            assert_eq!(locations.len(), 1);
            locations.remove(0)
        }
        GotoDefinitionResponse::Link(_) => panic!("expected location result"),
    }
}

fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    vize_s0::corsa_resolver::resolve_corsa_executable(
        vize_s0::corsa_resolver::CorsaResolveRequest {
            project_root: Some(workspace_root),
            ..Default::default()
        },
    )
    .ok()
}
