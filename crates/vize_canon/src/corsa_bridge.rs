//! Corsa bridge for the native TypeScript checker.
//!
//! This module keeps the rest of the workspace insulated from process spawning,
//! virtual document syncing, and the msgpack project-session details hidden
//! behind the bridge surface.

mod bridge;
mod script_document;
#[cfg(test)]
mod script_document_tests;
mod session;
mod types;
mod vue_dependencies;
mod vue_dependencies_alias;
mod vue_dependency_paths;
mod vue_dependency_specifiers;
mod vue_document;
#[cfg(test)]
mod vue_document_alias_tests;
// Every case links a workspace package through a directory symlink, which
// Windows refuses without Developer Mode or elevation.
#[cfg(all(test, unix))]
mod vue_document_package_tests;
#[cfg(test)]
mod vue_document_tests;
#[cfg(test)]
mod vue_project_mapping_tests;
mod worker;

pub use bridge::{BatchTypeChecker, CorsaBridge};
pub use types::{
    CorsaBridgeConfig, CorsaBridgeError, LspCompletionItem, LspCompletionList,
    LspCompletionResponse, LspDefinitionResponse, LspDiagnostic, LspDocumentation, LspHover,
    LspHoverContents, LspLocation, LspLocationLink, LspMarkedString, LspMarkupContent,
    LspParameterInformation, LspParameterLabel, LspPosition, LspRange, LspSignatureHelp,
    LspSignatureInformation, TypeCheckResult, VIRTUAL_URI_SCHEME,
};
pub use vue_document::{
    CorsaVueVirtualDependency, CorsaVueVirtualDocument, CorsaVueVirtualDocumentOptions,
};
pub(crate) use vue_document::{CorsaVueVirtualProject, build_vue_virtual_project};

#[cfg(test)]
mod tests {
    use super::{
        CorsaBridge, CorsaBridgeConfig, CorsaBridgeError, LspDiagnostic, LspPosition, LspRange,
        TypeCheckResult, VIRTUAL_URI_SCHEME, bridge,
    };
    use crate::file_uri::path_to_file_uri;
    use corsa::runtime::block_on;
    use std::path::{Path, PathBuf};
    use vize_carton::cstr;

    #[test]
    fn test_virtual_uri_format() {
        let name = "Component.vue.ts";
        let uri = cstr!("{VIRTUAL_URI_SCHEME}://{name}");
        assert_eq!(uri, "vize-virtual://Component.vue.ts");
    }

    #[test]
    fn test_type_check_result() {
        let mut result = TypeCheckResult::default();
        assert!(!result.has_errors());
        assert_eq!(result.error_count(), 0);

        result.diagnostics.push(LspDiagnostic {
            range: LspRange {
                start: LspPosition {
                    line: 0,
                    character: 0,
                },
                end: LspPosition {
                    line: 0,
                    character: 10,
                },
            },
            severity: Some(1),
            code: None,
            source: Some("ts".into()),
            message: "Type error".into(),
            related_information: None,
        });

        assert!(result.has_errors());
        assert_eq!(result.error_count(), 1);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn test_config_default() {
        let config = CorsaBridgeConfig::default();
        assert!(config.corsa_path.is_none());
        assert!(config.working_dir.is_none());
        assert_eq!(config.timeout_ms, 30000);
        assert!(!config.enable_profiling);
    }

    /// Writes a backend that accepts stdio and never answers — the shape
    /// #3376 describes, and the one a `timeout` combinator cannot rescue.
    ///
    /// `cat` drains the request stream so the writer never blocks, and the
    /// shell keeps the stdout pipe open without ever writing to it, so the
    /// reader sees neither a reply nor an EOF. Redirecting the *shell's*
    /// stdout (or `exec`ing) would close that pipe and surface a spawn error
    /// instead of the silence being reproduced here.
    #[cfg(unix)]
    fn write_hanging_backend(dir: &Path) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let script = dir.join("hanging-corsa");
        std::fs::write(&script, "#!/bin/sh\ncat > /dev/null\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        script
    }

    /// `timeout_ms` has to be asserted as behavior, not as a default value
    /// (#3376). Bridge IPC is synchronous, so no `timeout` combinator wrapped
    /// around a bridge call can enforce a deadline; before this the configured
    /// bound was read by nothing at all, and a silent backend blocked the
    /// caller until `corsa`'s own 30s transport backstop killed the session —
    /// per request, on the single thread that drives the whole LSP server.
    ///
    /// The assertion is on the outcome, not the clock: pre-fix this call
    /// answers `SpawnFailed` after `corsa` gives up, post-fix it answers
    /// `Timeout` because vize's own bound fired first.
    #[test]
    #[cfg(unix)]
    fn a_backend_that_never_answers_is_bounded_by_the_configured_timeout() {
        // Not cleaned up afterwards on purpose: the abandoned handshake still
        // owns the backend, and pulling the script out from under a process
        // that has not finished starting only produces exec noise. The pid
        // keys the directory and it is cleared on entry, so a run leaves at
        // most one of these behind.
        let dir = std::env::temp_dir()
            .join(cstr!("vize-corsa-bridge-bound-{}", std::process::id()).as_str());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let bridge = CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(write_hanging_backend(&dir)),
            working_dir: Some(dir.clone()),
            timeout_ms: 250,
            ..Default::default()
        });

        let first = block_on(bridge.spawn());
        // The abandoned handshake still owns the worker, so the retry must be
        // refused outright rather than pay the deadline a second time.
        let second = block_on(bridge.spawn());

        assert!(
            matches!(first, Err(CorsaBridgeError::Timeout)),
            "expected the configured bound to fire, got {first:?}"
        );
        assert!(
            matches!(second, Err(CorsaBridgeError::Timeout)),
            "expected the outstanding stall to be reported, got {second:?}"
        );
        assert!(!bridge.is_initialized());
    }

    #[test]
    fn normalizes_absolute_paths_with_file_uri_encoding() {
        let path = Path::new("/workspace/app=demo/src/App.vue.ts");

        assert_eq!(
            bridge::normalize_document_uri(path.to_str().unwrap()),
            path_to_file_uri(path)
        );
    }
}
