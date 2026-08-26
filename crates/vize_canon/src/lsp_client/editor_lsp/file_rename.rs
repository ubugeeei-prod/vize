//! Native `workspace/willRenameFiles` routing over the reusable editor LSP
//! session.

use serde_json::Value;
use vize_s0::String;

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

        let result =
            self.request_with_editor_lsp_recovery(|session| session.will_rename_files(renames));
        match result {
            Ok(edit) => {
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
