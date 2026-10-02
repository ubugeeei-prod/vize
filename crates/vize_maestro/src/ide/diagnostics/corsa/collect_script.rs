//! Type diagnostics for authored JavaScript and TypeScript documents.

use oxc_span::SourceType;
use tower_lsp::lsp_types::{Diagnostic, Url};
use vize_canon::{CorsaScriptVirtualDocumentRequest, CorsaVueVirtualDocumentOptions};

use super::super::{DiagnosticService, VirtualTsResult};
use super::collect::{CollectFailure, classify};
use super::collect_virtual::{
    CorsaDocument, assemble_script_corsa_diagnostics, fetch_finished_diagnostics,
};
use crate::server::ServerState;

pub(super) fn is_script_uri(uri: &Url) -> bool {
    matches!(
        uri.path().rsplit_once('.').map(|(_, extension)| extension),
        Some("js" | "mjs" | "cjs" | "ts" | "mts" | "cts")
    )
}

impl DiagnosticService {
    pub(super) async fn try_collect_corsa_script_diagnostics(
        state: &ServerState,
        uri: &Url,
    ) -> Result<Vec<Diagnostic>, CollectFailure> {
        let Some((revision, content)) = state
            .documents
            .get(uri)
            .map(|document| (document.revision(), document.text()))
        else {
            return Ok(vec![]);
        };
        let Ok(source_path) = uri.to_file_path() else {
            return Ok(vec![]);
        };
        let Ok(source_type) = SourceType::from_path(&source_path) else {
            return Ok(vec![]);
        };
        let Some(bridge) = state.get_corsa_bridge().await else {
            return Err(CollectFailure::Unavailable);
        };
        let cached_overlays = state.corsa_overlays();
        let overlays = cached_overlays
            .iter()
            .map(|(path, text)| (path.clone(), &**text))
            .collect::<Vec<_>>();
        let virtual_ts_options = state.virtual_ts_options();
        let opened = bridge
            .open_script_virtual_document_with_vue_dependencies(CorsaScriptVirtualDocumentRequest {
                source_path: &source_path,
                request_path: uri.as_str(),
                code: &content,
                source_type,
                options: CorsaVueVirtualDocumentOptions {
                    options_api: state.options_api_enabled(),
                    legacy_vue2: state.legacy_vue2_enabled(),
                    jsx_typecheck: state.jsx_typecheck_enabled(),
                    experimental_patterned_template: state.patterned_template_enabled(),
                    preserve_event_navigation: true,
                    dialect: state.type_checker_vue_version(),
                },
                overlays: &overlays,
                virtual_ts_options: &virtual_ts_options,
            })
            .await
            .map_err(|error| classify(&bridge, error))?;
        state.record_typecheck_dependencies(uri, revision, &opened.resolved_dependencies);
        let mut mappings = opened.mappings;
        if mappings.is_empty() {
            mappings.push(vize_canon::virtual_ts::VizeMapping {
                src_range: 0..content.len(),
                gen_range: 0..content.len(),
                sub_spans: Vec::new(),
            });
        }
        let document = CorsaDocument::from_result(VirtualTsResult {
            code: opened.code.to_string(),
            source_mappings: mappings,
            semantic_links: Vec::new(),
            import_source_map: opened.import_source_map,
        });
        let finished = fetch_finished_diagnostics(&bridge, &opened.request_uri, &document, 0)
            .await
            .map_err(|error| classify(&bridge, error))?;
        Ok(assemble_script_corsa_diagnostics(
            &content,
            &[document],
            finished,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::NumberOrString;
    use vize_l0::cstr;

    #[test]
    fn only_authored_plain_scripts_take_the_corsa_script_path() {
        for path in [
            "plain.ts",
            "plain.mts",
            "plain.cts",
            "plain.js",
            "plain.mjs",
            "plain.cjs",
        ] {
            assert!(is_script_uri(
                &Url::parse(&cstr!("file:///src/{path}")).unwrap()
            ));
        }
        for path in ["App.vue", "plain.tsx", "plain.jsx", "config.json"] {
            assert!(!is_script_uri(
                &Url::parse(&cstr!("file:///src/{path}")).unwrap()
            ));
        }
    }

    #[test]
    fn authored_ts_document_reports_type_error() {
        if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
            return;
        }
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap();
        let Ok(corsa_path) = vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                project_root: Some(workspace_root),
                ..Default::default()
            },
        ) else {
            return;
        };

        let project = tempfile::tempdir().unwrap();
        std::fs::write(
            project.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["*.ts"]}"#,
        ).unwrap();
        std::fs::write(
            project.path().join("vize.config.json"),
            serde_json::json!({
                "lsp": {"lint": false, "typecheck": true},
                "typeChecker": {"corsaPath": corsa_path}
            })
            .to_string(),
        )
        .unwrap();
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/lsp/plain-ts-typecheck/plain.ts.txt"
        ));
        let path = project.path().join("plain.ts");
        std::fs::write(&path, source).unwrap();
        let uri = Url::from_file_path(path).unwrap();
        let state = ServerState::new();
        state.load_lsp_config(project.path());
        state.set_workspace_root(project.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.into(), 1, "typescript".into());
        let diagnostics = crate::runtime::block_on(DiagnosticService::collect_async(&state, &uri));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| matches!(diagnostic.code, Some(NumberOrString::Number(2322)))),
            "expected TS2322 from a plain .ts document: {diagnostics:#?}"
        );
        assert!(
            !diagnostics.iter().any(|diagnostic| matches!(
                diagnostic.code.as_ref(),
                Some(NumberOrString::String(code)) if code == "typecheck-unavailable"
            )),
            "unexpected backend-unavailable hint: {diagnostics:#?}"
        );
    }
}
