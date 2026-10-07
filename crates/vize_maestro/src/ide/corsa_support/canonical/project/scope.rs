use tower_lsp::lsp_types::Url;

use crate::ide::IdeContext;

/// Keep configured-project operations out of unrelated workspace packages.
/// If the governing config is missing, preserve the inferred-project fallback
/// and search the discovered workspace surface. A configured project that does
/// not own the query must not donate any workspace sources to that query.
pub(in super::super) fn same_typescript_project(
    ctx: &IdeContext<'_>,
    sources: Vec<(Url, std::string::String)>,
) -> Vec<(Url, std::string::String)> {
    let Some(source_path) = ctx.uri.to_file_path().ok() else {
        return sources;
    };
    let Some(tsconfig) = source_path
        .ancestors()
        .skip(1)
        .flat_map(|directory| {
            [
                directory.join("tsconfig.json"),
                directory.join("jsconfig.json"),
            ]
        })
        .find(|candidate| candidate.is_file())
    else {
        return sources;
    };
    let mut ownership = vize_canon::batch::TsconfigOwnershipCache::default();
    let projects = ownership.project_paths(&tsconfig);
    let owns = |ownership: &mut vize_canon::batch::TsconfigOwnershipCache,
                path: &std::path::Path| {
        projects.iter().any(|project| {
            ownership.project_owns_source(
                project,
                path,
                if matches!(
                    path.extension().and_then(|ext| ext.to_str()),
                    Some("js" | "jsx" | "mjs" | "cjs")
                ) {
                    vize_canon::batch::TsconfigSourceKind::JavaScript
                } else {
                    vize_canon::batch::TsconfigSourceKind::Typed
                },
            )
        })
    };
    if !owns(&mut ownership, &source_path) {
        return Vec::new();
    }
    sources
        .into_iter()
        .filter(|(uri, _)| {
            uri.to_file_path()
                .ok()
                .is_some_and(|path| owns(&mut ownership, &path))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tower_lsp::lsp_types::Url;

    use super::same_typescript_project;
    use crate::ide::IdeContext;
    use crate::server::ServerState;

    #[test]
    fn configured_project_does_not_feed_workspace_sources_to_excluded_query() {
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        let ignored = project.path().join("ignored");
        fs::create_dir_all(&src).expect("src directory");
        fs::create_dir_all(&ignored).expect("ignored directory");
        fs::write(
            project.path().join("tsconfig.json"),
            r#"{ "include": ["src/**/*"] }"#,
        )
        .expect("tsconfig");

        let included_path = src.join("Included.vue");
        let excluded_path = ignored.join("Excluded.vue");
        let source = "<script setup lang=\"ts\">const shared = 1</script>";
        fs::write(&included_path, source).expect("included component");
        fs::write(&excluded_path, source).expect("excluded component");
        let included_uri = Url::from_file_path(included_path).expect("included URI");
        let excluded_uri = Url::from_file_path(excluded_path).expect("excluded URI");

        let state = ServerState::new();
        state.documents.open(
            excluded_uri.clone(),
            source.to_string(),
            1,
            "vue".to_string(),
        );
        let ctx = IdeContext::new(&state, &excluded_uri, 0).expect("excluded query context");

        let filtered = same_typescript_project(
            &ctx,
            vec![
                (included_uri, source.to_string()),
                (excluded_uri.clone(), source.to_string()),
            ],
        );

        assert!(
            filtered.is_empty(),
            "a query excluded by the governing tsconfig must not search its workspace surface",
        );
    }
}

#[cfg(test)]
mod javascript_admission_tests {
    use super::same_typescript_project;
    use crate::{ide::IdeContext, server::ServerState};
    use tower_lsp::lsp_types::Url;

    #[test]
    fn configured_js_sources_honor_authored_allow_js_and_project_boundaries() {
        let root = tempfile::tempdir().unwrap();
        let src = root.path().join("src");
        std::fs::create_dir_all(&src).unwrap();
        let uri = Url::from_file_path(src.join("App.vue")).unwrap();
        let script_uri = Url::from_file_path(src.join("importer.js")).unwrap();
        let outside = Url::from_file_path(root.path().join("outside.ts")).unwrap();
        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), "<template />".into(), 1, "vue".into());
        let ctx = IdeContext::new(&state, &uri, 0).unwrap();
        for allow_js in [false, true] {
            std::fs::write(
                root.path().join("jsconfig.json"),
                serde_json::json!({
                    "compilerOptions": { "allowJs": allow_js }, "include": ["src/**/*"]
                })
                .to_string(),
            )
            .unwrap();
            let sources = vec![
                (script_uri.clone(), "export const value = 1;".into()),
                (outside.clone(), "export const foreign = 2;".into()),
            ];
            let expected = if allow_js {
                vec![(script_uri.clone(), "export const value = 1;".into())]
            } else {
                Vec::new()
            };
            assert_eq!(same_typescript_project(&ctx, sources), expected);
        }
    }
}
