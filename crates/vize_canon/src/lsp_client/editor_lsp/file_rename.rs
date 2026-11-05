//! Native `workspace/willRenameFiles` routing over the reusable editor LSP
//! session.

use serde_json::Value;
use vize_l0::{String, cstr};

use super::CorsaProjectClient;

impl CorsaProjectClient {
    pub(in crate::lsp_client) fn will_rename_files_via_editor_lsp(
        &mut self,
        renames: &[(&str, &str)],
    ) -> Result<Option<Value>, String> {
        if renames.is_empty()
            || self.editor_lsp_will_rename_supported == Some(false)
            || !renames.iter().all(|(old_uri, new_uri)| {
                is_script_document_uri(old_uri) && is_script_document_uri(new_uri)
            })
        {
            return Ok(None);
        }

        let native = renames
            .iter()
            .map(|(old, new)| {
                Ok((
                    self.rename_document_uri(old)?,
                    self.rename_document_uri(new)?,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let native_renames = native
            .iter()
            .map(|(old, new)| (old.as_str(), new.as_str()))
            .collect::<Vec<_>>();
        let result = self
            .request_with_editor_lsp_recovery(|session| session.will_rename_files(&native_renames));
        match result {
            Ok(mut edit) => {
                if let Some(value) = &mut edit {
                    self.remap_result_uris(value);
                }
                self.editor_lsp_will_rename_supported = Some(true);
                Ok(edit)
            }
            Err(error) if editor_lsp_will_rename_error_is_unsupported(&error) => {
                self.editor_lsp_will_rename_supported = Some(false);
                Ok(None)
            }
            Err(error) => Err(error),
        }
    }

    fn rename_document_uri(&mut self, uri: &str) -> Result<String, String> {
        if self.materialized_project_session {
            self.materialized_session_document_uri(uri)
                .ok_or_else(|| cstr!("Cannot materialize rename document: {uri}"))
        } else {
            Ok(self.editor_document_uri(uri))
        }
    }
}

/// Whether tsgo can parse the renamed identity.
///
/// The runtime derives a `ScriptKind` from the file extension while building
/// the program that answers the rename, and it panics with `ScriptKind must be
/// specified when parsing source file` on an extension it does not know — `.vue`
/// above all. That crash now takes down the session shared with hover,
/// completion, and diagnostics, so a rename the runtime cannot represent must
/// never reach it; the caller falls back to the import scanner instead.
fn is_script_document_uri(uri: &str) -> bool {
    [".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs"]
        .iter()
        .any(|extension| uri.ends_with(extension))
}

fn editor_lsp_will_rename_error_is_unsupported(error: &str) -> bool {
    error.contains("workspace/willRenameFiles")
        && (error.contains("unknown method")
            || error.contains("method not found")
            || error.contains("InvalidRequest")
            || error.contains("Unsupported")
            || error.contains("unsupported")
            || error.contains("not supported"))
}

#[cfg(test)]
mod tests {
    use super::is_script_document_uri;

    #[cfg(unix)]
    #[test]
    fn materialized_renames_share_one_native_namespace_and_keep_each_caller_context() {
        use crate::file_uri::path_to_file_uri;
        use crate::lsp_client::CorsaProjectClient;
        use serde_json::{Value, json};
        use std::{fs, os::unix::fs::PermissionsExt};

        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let workspace = root.join("workspace");
        let linked = root.join("linked");
        fs::create_dir(&workspace).unwrap();
        std::os::unix::fs::symlink(&workspace, &linked).unwrap();
        for name in ["first.ts", "second.ts"] {
            fs::write(workspace.join(name), "export const value = 1;").unwrap();
        }
        let peer = root.join("rename-peer.ts");
        let transcript = root.join("rename-packets.jsonl");
        let original = include_str!("native_shutdown/peer.ts.txt");
        let anchor = "} else if (request.method === \"textDocument/hover\") {";
        assert_eq!(original.matches(anchor).count(), 1);
        let handler = r#"} else if (request.method === "workspace/willRenameFiles") {
      const { files } = request.params as { files: Array<{ oldUri: string; newUri: string }> };
      await respond(request, { changes: Object.fromEntries(files.map(({ oldUri }) => [oldUri, [{ newText: oldUri }]])) });
    } else if (request.method === "textDocument/hover") {"#;
        fs::write(
            &peer,
            original
                .replace("LOG_PATH", &serde_json::to_string(&transcript).unwrap())
                .replace("REPLY_MODE", "\"ok\"")
                .replace(anchor, handler),
        )
        .unwrap();
        fs::set_permissions(&peer, fs::Permissions::from_mode(0o755)).unwrap();
        let mut client = CorsaProjectClient::batch_editor_peer_for_test(&peer, &workspace).unwrap();
        client.materialized_project_session = true;
        let old_first = path_to_file_uri(&linked.join("first.ts"));
        let new_first = path_to_file_uri(&linked.join("first-renamed.ts"));
        let old_second = path_to_file_uri(&workspace.join("second.ts"));
        let new_second = path_to_file_uri(&workspace.join("second-renamed.ts"));
        let native = [
            "first.ts",
            "first-renamed.ts",
            "second.ts",
            "second-renamed.ts",
        ]
        .map(|name| path_to_file_uri(&client.overlay_storage.root().join(name)));
        let edit = client
            .will_rename_files_via_editor_lsp(&[
                (&old_first, &new_first),
                (&old_second, &new_second),
            ])
            .unwrap();
        client.shutdown().unwrap();
        let packets = fs::read_to_string(&transcript)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        let requests = packets
            .iter()
            .filter(|packet| packet["direction"] == "request")
            .map(|packet| &packet["wholePacket"])
            .filter(|packet| packet["method"] == "workspace/willRenameFiles")
            .map(|packet| packet["params"].clone())
            .collect::<Vec<_>>();
        assert_eq!(
            requests,
            vec![json!({"files":[
                {"oldUri":native[0],"newUri":native[1]},
                {"oldUri":native[2],"newUri":native[3]},
            ]})]
        );
        assert_eq!(
            edit,
            Some(json!({"changes":{
                old_first:[{"newText":native[0]}], old_second:[{"newText":native[2]}],
            }}))
        );
        assert_eq!(packets.last(), Some(&json!({"event":"stdin-eof"})));
    }

    #[test]
    fn script_identities_route_through_the_native_rename() {
        for uri in [
            "file:///workspace/src/module.ts",
            "file:///workspace/src/View.tsx",
            "file:///workspace/src/loader.mts",
            "file:///workspace/src/loader.cts",
            "file:///workspace/src/script.js",
            "file:///workspace/src/View.jsx",
            "file:///workspace/src/script.mjs",
            "file:///workspace/src/script.cjs",
            "file:///workspace/src/App.vue.ts",
        ] {
            assert!(is_script_document_uri(uri), "{uri}");
        }
    }

    #[test]
    fn identities_tsgo_cannot_parse_stay_on_the_import_scanner() {
        for uri in [
            "file:///workspace/src/App.vue",
            "file:///workspace/src/page.html",
            "file:///workspace/src/styles.css",
            "file:///workspace/package.json",
            "file:///workspace/src/component",
        ] {
            assert!(!is_script_document_uri(uri), "{uri}");
        }
    }
}
