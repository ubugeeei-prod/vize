//! Packaged SFCs are typed by their shipped declarations, not compiled JavaScript.

use std::io;
use std::path::Path;

use vize_canon::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperTransform,
};

/// Read the declaration shipped beside a packaged `.vue` file.
///
/// Application source is deliberately excluded: an adjacent declaration can
/// be stale while the author edits the SFC. Published packages are different;
/// their declarations are the package's public type surface, while their Vue
/// files may contain compiled JavaScript with no named types or slot metadata.
pub(super) fn read_shipped_declaration(
    file: &Path,
    skip_lib_check: bool,
) -> io::Result<Option<ContentMapperTransform>> {
    if file.extension().is_none_or(|extension| extension != "vue")
        || !file
            .components()
            .any(|component| component.as_os_str() == "node_modules")
    {
        return Ok(None);
    }

    for declaration in [
        file.with_extension("d.vue.ts"),
        file.with_extension("vue.d.ts"),
    ] {
        let text = match std::fs::read_to_string(declaration) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        let text = text.trim_start_matches('\u{feff}');
        // The host parses mapper output as source rather than as a declaration
        // file. Reproduce skipLibCheck for this virtual declaration only;
        // diagnostics in importing application files stay enabled.
        let diagnostic_directives = skip_lib_check.then(|| ContentMapperDiagnosticDirectives {
            unused_expect_directive_diagnostics: Vec::new(),
            directives: vec![ContentMapperDiagnosticDirective([
                0,
                0,
                0,
                text.len(),
                0,
                0,
            ])],
        });
        return Ok(Some(ContentMapperTransform {
            text: text.into(),
            extension: ".ts",
            mappings: Vec::new(),
            semantic_links: Vec::new(),
            diagnostic_directives,
            diagnostics: Vec::new(),
        }));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::read_shipped_declaration;
    use crate::commands::content_mapper::test_support::{
        exchange, frames, initialize_request, open_project_request, transform_request,
    };
    use serde_json::{Value, json};
    use std::path::Path;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn packaged_declaration_order_and_diagnostic_policy_are_explicit() {
        let root = tempfile::tempdir().unwrap();
        let component = root.path().join("node_modules/ui/Badge.vue");
        write(
            &component.with_extension("vue.d.ts"),
            "export type Fallback = 1;\n",
        );
        write(
            &component.with_extension("d.vue.ts"),
            "\u{feff}export type Preferred = 1;\n",
        );

        let checked = read_shipped_declaration(&component, false)
            .unwrap()
            .unwrap();
        assert_eq!(checked.text, "export type Preferred = 1;\n");
        assert_eq!(checked.extension, ".ts");
        assert_eq!(checked.diagnostic_directives, None);

        let skipped = read_shipped_declaration(&component, true).unwrap().unwrap();
        let table = skipped.diagnostic_directives.unwrap();
        assert_eq!(table.unused_expect_directive_diagnostics, []);
        assert_eq!(table.directives[0].0, [0, 0, 0, skipped.text.len(), 0, 0]);
    }

    #[test]
    fn local_sources_and_packages_without_declarations_fall_through() {
        let root = tempfile::tempdir().unwrap();
        let local = root.path().join("src/Badge.vue");
        write(
            &local.with_extension("d.vue.ts"),
            "export type Stale = 1;\n",
        );
        assert!(read_shipped_declaration(&local, true).unwrap().is_none());

        let packaged = root.path().join("node_modules/ui/Badge.vue");
        write(&packaged, "<template />\n");
        assert!(read_shipped_declaration(&packaged, true).unwrap().is_none());
    }

    #[test]
    fn server_returns_the_declaration_and_scopes_skip_lib_check_to_it() {
        let root = tempfile::tempdir().unwrap();
        let component = root.path().join("node_modules/ui/Badge.vue");
        let declaration = "export interface BadgeProps { label: string }\n";
        write(&component.with_extension("d.vue.ts"), declaration);

        let transform = |id, skip_lib_check| {
            let mut request = transform_request(id, "<script>export default {}</script>");
            request["params"]["fileName"] = json!(component);
            exchange(&frames(&[
                initialize_request(),
                open_project_request(2, Value::Null, json!({ "skipLibCheck": skip_lib_check })),
                request,
            ]))
        };

        let skipped = transform(3, true);
        let result = &skipped[2]["result"];
        assert_eq!(result["text"], declaration);
        assert_eq!(result["extension"], ".ts");
        assert_eq!(result["mappings"], json!([]));
        assert_eq!(
            result["diagnosticDirectives"]["directives"],
            json!([[0, 0, 0, declaration.len(), 0]])
        );

        let checked = transform(3, false);
        assert_eq!(checked[2]["result"]["text"], declaration);
        assert!(checked[2]["result"].get("diagnosticDirectives").is_none());
    }
}
