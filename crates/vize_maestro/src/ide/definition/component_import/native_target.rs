//! A component's package branch selected by the current native project.

use std::path::PathBuf;

use tower_lsp::lsp_types::GotoDefinitionResponse;
use vize_canon::CorsaBridge;

use crate::ide::IdeContext;

/// The synchronous path intentionally refuses conditional/legacy ambiguity.
/// Ask the same native module-definition query as the import specifier, then
/// inspect only the selected authored module. No manifest candidate is chosen here.
pub(crate) async fn resolve_native_component_file(
    ctx: &IdeContext<'_>,
    component_name: &str,
    bridge: &CorsaBridge,
) -> Option<PathBuf> {
    if !bridge.is_initialized() {
        return None;
    }
    let import = super::find_component_import(ctx, component_name)
        .or_else(|| super::find_registered_component_import(ctx, component_name))?;
    let definition = super::super::module_specifier::definition_at_offset_with_corsa(
        ctx,
        bridge,
        import.specifier_offset,
    )
    .await?;
    let locations = match definition {
        GotoDefinitionResponse::Scalar(location) => vec![location],
        GotoDefinitionResponse::Array(locations) => locations,
        GotoDefinitionResponse::Link(_) => return None,
    };
    let mut targets = locations
        .into_iter()
        .filter_map(|location| {
            let module = location.uri.to_file_path().ok()?;
            if super::is_vue_path(&module) {
                (import.imported_name == "default").then_some(module)
            } else {
                super::resolve_exported_component(&module, &import.imported_name, 0)
            }
        })
        .collect::<Vec<_>>();
    targets.sort();
    targets.dedup();
    (targets.len() == 1).then(|| targets.remove(0))
}

#[cfg(test)]
mod tests;
