use super::{ServerState, open_vue_importers};
use tower_lsp::lsp_types::Url;

/// Return open Vue documents whose diagnostics may change with `dependency`.
///
/// Direct source imports use the reverse index. Declaration files can affect a
/// Vue document through an arbitrary package/relative import chain, so refresh
/// every open Vue document for `.d.ts`, `.d.mts`, and `.d.cts` edits. This is
/// bounded by editor-visible documents and avoids a workspace-wide rescan.
pub(in crate::server) fn open_vue_dependents(state: &ServerState, dependency: &Url) -> Vec<Url> {
    let mut dependents = open_vue_importers(state, dependency);
    if is_declaration_uri(dependency) {
        dependents.extend(
            state
                .documents
                .iter()
                .filter(|document| document.key().path().ends_with(".vue"))
                .map(|document| document.key().clone()),
        );
    }
    dependents.sort();
    dependents.dedup();
    dependents
}

fn is_declaration_uri(uri: &Url) -> bool {
    let path = uri.path();
    path.ends_with(".d.ts") || path.ends_with(".d.mts") || path.ends_with(".d.cts")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_methods)]

    use super::{ServerState, Url, open_vue_dependents};

    #[test]
    fn declaration_edits_refresh_all_open_vue_documents_only() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        let declaration_dir = dir.path().join("node_modules/pkg");
        let direct_dependency = src.join("direct.ts");
        let parent = src.join("Parent.vue");
        let unrelated = src.join("Unrelated.vue");
        std::fs::create_dir_all(&declaration_dir).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(&direct_dependency, "export const direct = true\n").unwrap();
        std::fs::write(&parent, "<template />\n").unwrap();
        std::fs::write(&unrelated, "<template />\n").unwrap();

        let direct_uri = Url::from_file_path(&direct_dependency).unwrap();
        let parent_uri = Url::from_file_path(&parent).unwrap();
        let unrelated_uri = Url::from_file_path(&unrelated).unwrap();
        let state = ServerState::new();
        let parent_source = "<script setup lang=\"ts\">import { direct } from './direct'</script>";
        state.documents.open(
            parent_uri.clone(),
            parent_source.to_string(),
            1,
            "vue".to_string(),
        );
        state.documents.open(
            unrelated_uri.clone(),
            "<template />".to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(&parent_uri, parent_source);
        state.update_virtual_docs(&unrelated_uri, "<template />");

        assert_eq!(
            open_vue_dependents(&state, &direct_uri),
            vec![parent_uri.clone()],
        );
        for declaration_name in ["theme.d.ts", "theme.d.mts", "theme.d.cts"] {
            let declaration = declaration_dir.join(declaration_name);
            std::fs::write(&declaration, "export interface Theme {}\n").unwrap();
            let declaration_uri = Url::from_file_path(&declaration).unwrap();
            assert_eq!(
                open_vue_dependents(&state, &declaration_uri),
                vec![parent_uri.clone(), unrelated_uri.clone()],
                "{declaration_name}",
            );
        }
    }
}
