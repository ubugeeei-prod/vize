use std::fs;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};

use super::DefinitionService;
use crate::{ide::IdeContext, server::ServerState};

mod component_props;
mod imported_symbols;
mod options_methods;
mod package_specifiers;
mod script_unicode;
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
    crate::test_support::tsgo()
}
