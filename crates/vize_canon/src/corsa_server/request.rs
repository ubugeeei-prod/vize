//! JSON-RPC dispatch and one persistent Vue check request.

use std::path::PathBuf;

use vize_carton::{String, cstr};

use super::{
    CheckParams, CheckResult, CorsaServer, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    diagnostics, sfc_semantics,
};

impl CorsaServer {
    pub(super) fn handle_request(&mut self, input: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(input) {
            Ok(request) => request,
            Err(error) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: cstr!("Parse error: {error}"),
                        data: None,
                    }),
                };
            }
        };

        match request.method.as_str() {
            "check" => self.handle_check(request.id, request.params),
            "check-project" => self.handle_project_check(request.id, request.params),
            "session-stats" => JsonRpcResponse {
                jsonrpc: "2.0",
                id: request.id,
                result: Some(serde_json::json!({
                    "projectInits": self.project_inits(),
                    "liveSessions": self.live_sessions(),
                })),
                error: None,
            },
            "shutdown" => {
                self.running
                    .store(false, std::sync::atomic::Ordering::SeqCst);
                JsonRpcResponse {
                    jsonrpc: "2.0",
                    id: request.id,
                    result: Some(serde_json::json!({"status": "shutdown"})),
                    error: None,
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0",
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: cstr!("Method not found: {}", request.method),
                    data: None,
                }),
            },
        }
    }

    fn handle_check(&mut self, id: Option<u64>, params: serde_json::Value) -> JsonRpcResponse {
        let params: CheckParams = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(error) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0",
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: cstr!("Invalid params: {error}"),
                        data: None,
                    }),
                };
            }
        };

        match self.check_vue_sfc(&params.uri, &params.content, &params.flags) {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: error,
                    data: None,
                }),
            },
        }
    }

    fn check_vue_sfc(
        &mut self,
        uri: &str,
        content: &str,
        flags: &str,
    ) -> Result<CheckResult, String> {
        use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

        let working_dir = self.working_dir();
        let source_path =
            sfc_semantics::uri_to_path(uri, &working_dir).unwrap_or_else(|| PathBuf::from(uri));
        let virtual_ts_options = crate::virtual_ts::VirtualTsOptions::default();
        let project =
            crate::corsa_bridge::build_vue_virtual_project_with_overlays_and_options_and_package_routes(
                &source_path,
                content,
                Default::default(),
                &[],
                crate::corsa_bridge::CorsaProjectEnvironment {
                    virtual_ts_options: &virtual_ts_options,
                    package_routes: &self.package_route_resolver,
                    project_root: Some(&working_dir),
                    tsconfig_path: None,
                    editor_session: &self.editor_session,
                },
            )
            .map_err(|error| cstr!("Failed to generate virtual TS: {error}"))?;
        let virtual_ts = project.host.code.clone();
        self.cache.insert(uri.into(), virtual_ts.clone());

        let descriptor = parse_sfc(
            content,
            SfcParseOptions {
                filename: uri.into(),
                ..Default::default()
            },
        )
        .map_err(|error| cstr!("Failed to parse SFC: {}", error.message))?;

        let mut diagnostics = self.run_corsa(&project, content, &source_path, flags)?;
        diagnostics.extend(sfc_semantics::collect_sfc_compile_diagnostic(
            uri,
            content,
            &descriptor,
        ));
        diagnostics::dedup_diagnostics(&mut diagnostics);
        let error_count = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == "error")
            .count();

        Ok(CheckResult {
            diagnostics,
            virtual_ts,
            error_count,
        })
    }

    pub(super) fn working_dir(&self) -> PathBuf {
        self.config
            .working_dir
            .as_deref()
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::CorsaServer;
    use crate::corsa_server::ServerConfig;

    #[test]
    fn persistent_check_server_refreshes_package_target_create_and_delete() {
        let Some(corsa_path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) else {
            return;
        };
        if !corsa_path.is_file() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("app");
        let host = app.join("src/Host.vue");
        let package = app.join("node_modules/@scope/dynamic");
        write(
            &app.join("tsconfig.json"),
            include_str!(
                "../../../../tests/_fixtures/differential/typechecker/package-target-deletion-refresh/tsconfig.json.txt"
            ),
        );
        write(
            &package.join("package.json"),
            include_str!(
                "../../../../tests/_fixtures/differential/typechecker/package-target-deletion-refresh/package.json.txt"
            ),
        );
        let source = include_str!(
            "../../../../tests/_fixtures/differential/typechecker/package-target-deletion-refresh/Host.vue.txt"
        );
        write(&host, source);
        install_runtime_stubs(&app);
        let mut server = CorsaServer::with_config(ServerConfig {
            corsa_path: Some(corsa_path.to_string_lossy().into_owned().into()),
            working_dir: Some(app.to_string_lossy().into_owned().into()),
            ..ServerConfig::default()
        });
        let uri = crate::file_uri::path_to_file_uri(&host);

        let missing = server.check_vue_sfc(&uri, source, "").unwrap();
        assert!(has_code(&missing, "TS2307"));

        write(
            &package.join("Widget.vue"),
            "<script setup lang=\"ts\">defineProps<{ created: true }>()</script>\n",
        );
        let created = server.check_vue_sfc(&uri, source, "").unwrap();
        assert!(!has_code(&created, "TS2307"));

        std::fs::remove_file(package.join("Widget.vue")).unwrap();
        let deleted = server.check_vue_sfc(&uri, source, "").unwrap();
        assert!(has_code(&deleted, "TS2307"));

        write(
            &package.join("Widget.vue"),
            "<script setup lang=\"ts\">defineProps<{ repaired: true }>()</script>\n",
        );
        let repaired = server.check_vue_sfc(&uri, source, "").unwrap();
        assert!(!has_code(&repaired, "TS2307"), "{repaired:#?}");
        let unsaved = source.replace(
            "void Widget",
            "const authored: string = 42\nvoid Widget\nvoid authored",
        );
        let edited = server.check_vue_sfc(&uri, &unsaved, "").unwrap();
        assert!(has_code(&edited, "TS2322"), "{edited:#?}");
        assert!(!has_code(&edited, "TS2307"), "{edited:#?}");
        std::fs::remove_file(package.join("Widget.vue")).unwrap();
        let edited_deleted = server.check_vue_sfc(&uri, &unsaved, "").unwrap();
        assert!(has_code(&edited_deleted, "TS2307"), "{edited_deleted:#?}");
        assert!(has_code(&edited_deleted, "TS2322"), "{edited_deleted:#?}");
        write(
            &package.join("Widget.vue"),
            "<script setup lang=\"ts\">defineProps<{ repaired: true }>()</script>\n",
        );
        let edited_repaired = server.check_vue_sfc(&uri, &unsaved, "").unwrap();
        assert!(
            !has_code(&edited_repaired, "TS2307"),
            "{edited_repaired:#?}"
        );
        assert!(has_code(&edited_repaired, "TS2322"), "{edited_repaired:#?}");
        assert_eq!(std::fs::read_to_string(&host).unwrap(), source);
    }

    #[test]
    fn deleting_one_package_root_preserves_other_root_types_and_unsaved_hosts() {
        let Some(corsa_path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) else {
            return;
        };
        if !corsa_path.is_file() {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let mut projects = Vec::new();
        for (name, prop, value) in [
            ("first", "alpha: string", "{ alpha: 'ok' }"),
            ("second", "beta: number", "{ beta: 'wrong' }"),
        ] {
            let app = root.path().join(name);
            let host = app.join("src/Host.vue");
            let target = app.join("node_modules/@scope/shared/Widget.vue");
            write(
                &app.join("tsconfig.json"),
                r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler","allowArbitraryExtensions":true}}"#,
            );
            write(
                &target.parent().unwrap().join("package.json"),
                r#"{"name":"@scope/shared","version":"1.0.0","exports":"./Widget.vue"}"#,
            );
            let component =
                format!("<script setup lang=\"ts\">defineProps<{{ {prop} }}>()</script>\n");
            write(&target, &component);
            let disk = "<script setup lang=\"ts\">\nimport Widget from '@scope/shared'\nvoid Widget\n</script>\n";
            write(&host, disk);
            let source = disk.replace(
                "void Widget",
                &format!(
                    "const props: InstanceType<typeof Widget>['$props'] = {value}\nvoid props"
                ),
            );
            install_runtime_stubs(&app);
            let server = CorsaServer::with_config(ServerConfig {
                corsa_path: Some(corsa_path.to_string_lossy().into_owned().into()),
                working_dir: Some(app.to_string_lossy().into_owned().into()),
                ..ServerConfig::default()
            });
            projects.push((server, host, target, component, source));
        }
        let (mut first, first_host, first_target, component, first_source) = projects.remove(0);
        let (mut second, second_host, _, _, second_source) = projects.remove(0);
        let first_uri = crate::file_uri::path_to_file_uri(&first_host);
        let second_uri = crate::file_uri::path_to_file_uri(&second_host);
        let first_valid = first.check_vue_sfc(&first_uri, &first_source, "").unwrap();
        assert!(first_valid.diagnostics.is_empty(), "{first_valid:#?}");
        let second_invalid = second
            .check_vue_sfc(&second_uri, &second_source, "")
            .unwrap();
        assert!(has_code(&second_invalid, "TS2322"), "{second_invalid:#?}");
        assert!(!has_code(&second_invalid, "TS2307"), "{second_invalid:#?}");
        std::fs::remove_file(&first_target).unwrap();
        let first_deleted = first.check_vue_sfc(&first_uri, &first_source, "").unwrap();
        assert!(has_code(&first_deleted, "TS2307"), "{first_deleted:#?}");
        let second_unchanged = second
            .check_vue_sfc(&second_uri, &second_source, "")
            .unwrap();
        assert_eq!(
            serde_json::to_value(&second_unchanged.diagnostics).unwrap(),
            serde_json::to_value(&second_invalid.diagnostics).unwrap()
        );
        write(&first_target, &component);
        let first_repaired = first.check_vue_sfc(&first_uri, &first_source, "").unwrap();
        assert!(first_repaired.diagnostics.is_empty(), "{first_repaired:#?}");
        let second_still_invalid = second
            .check_vue_sfc(&second_uri, &second_source, "")
            .unwrap();
        assert_eq!(
            serde_json::to_value(&second_still_invalid.diagnostics).unwrap(),
            serde_json::to_value(&second_invalid.diagnostics).unwrap()
        );
        let disk = "<script setup lang=\"ts\">\nimport Widget from '@scope/shared'\nvoid Widget\n</script>\n";
        for (host, source) in [(&first_host, &first_source), (&second_host, &second_source)] {
            assert_eq!(std::fs::read_to_string(host).unwrap(), disk);
            assert_ne!(source, disk);
        }
    }

    fn has_code(result: &crate::corsa_server::CheckResult, code: &str) -> bool {
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_deref() == Some(code))
    }

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn install_runtime_stubs(project_root: &Path) {
        let node_modules = project_root.join("node_modules");
        crate::batch::write_vue_facade(&node_modules).unwrap();
        let runtime_dom = node_modules.join("@vue/runtime-dom");
        write(
            &runtime_dom.join("package.json"),
            r#"{"name":"@vue/runtime-dom","types":"index.d.ts"}"#,
        );
        write(
            &runtime_dom.join("index.d.ts"),
            crate::batch::VUE_RUNTIME_DOM_STUB_TYPES,
        );
        let vite = node_modules.join("vite");
        write(
            &vite.join("package.json"),
            r#"{"name":"vite","exports":{"./client":{"types":"./client.d.ts"}}}"#,
        );
        write(&vite.join("client.d.ts"), "export {};\n");
    }
}
