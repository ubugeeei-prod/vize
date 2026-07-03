use std::{fs, path::Path};

pub(in crate::commands::check::runner) fn is_suppressed_false_positive(
    diagnostic: &vize_canon::BatchDiagnostic,
) -> bool {
    is_nitro_import_meta_conflict(diagnostic)
        || is_vue_wildcard_component_duplicate(diagnostic)
        || is_nuxt_bridge_injected_property_duplicate(diagnostic)
}

fn is_nitro_import_meta_conflict(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2320)
        && diagnostic
            .message
            .contains("Interface 'ImportMeta' cannot simultaneously extend types")
        && diagnostic.message.contains("NitroStaticBuildFlags")
        && diagnostic.message.contains("NitroImportMeta")
}

fn is_vue_wildcard_component_duplicate(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2300)
        && diagnostic
            .message
            .contains("Duplicate identifier 'component'")
        && declaration_source_contains(
            &diagnostic.file,
            &["declare module \"*.vue\"", "declare module '*.vue'"],
        )
}

fn is_nuxt_bridge_injected_property_duplicate(diagnostic: &vize_canon::BatchDiagnostic) -> bool {
    diagnostic.code == Some(2717)
        && diagnostic
            .message
            .contains("Subsequent property declarations must have the same type")
        && diagnostic.message.contains("Property '$")
        && diagnostic.message.contains("must be of type 'any'")
        && declaration_source_contains(
            &diagnostic.file,
            &[
                "declare module \"@nuxt/bridge-schema\"",
                "declare module '@nuxt/bridge-schema'",
            ],
        )
}

fn declaration_source_contains(path: &Path, needles: &[&str]) -> bool {
    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts"))
    {
        return false;
    }
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    needles.iter().any(|needle| source.contains(needle))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use vize_canon::BatchDiagnostic;

    use super::is_suppressed_false_positive;

    fn diagnostic(file: PathBuf, code: u32, message: &str) -> BatchDiagnostic {
        BatchDiagnostic {
            file,
            line: 0,
            column: 0,
            message: message.into(),
            code: Some(code),
            severity: 1,
            block_type: None,
        }
    }

    #[test]
    fn suppresses_project_vue_wildcard_component_duplicates() {
        let temp = tempfile::tempdir().unwrap();
        let shim = temp.path().join("ts-shim.d.ts");
        std::fs::write(
            &shim,
            "declare module '*.vue' {\n  import Vue from 'vue';\n  export default Vue;\n}\n",
        )
        .unwrap();

        assert!(is_suppressed_false_positive(&diagnostic(
            shim,
            2300,
            "Duplicate identifier 'component'.",
        )));
    }

    #[test]
    fn suppresses_nuxt_bridge_injection_duplicates_against_any() {
        let temp = tempfile::tempdir().unwrap();
        let shim = temp.path().join("gtag.d.ts");
        std::fs::write(
            &shim,
            "declare module '@nuxt/bridge-schema' {\n  interface Context { $gtag: Gtag.Gtag; }\n}\n",
        )
        .unwrap();

        assert!(is_suppressed_false_positive(&diagnostic(
            shim,
            2717,
            "Subsequent property declarations must have the same type.  Property '$gtag' must be of type 'any', but here has type 'Gtag'.",
        )));
    }

    #[test]
    fn keeps_unrelated_declaration_conflicts_visible() {
        let temp = tempfile::tempdir().unwrap();
        let declaration = temp.path().join("conflict.d.ts");
        std::fs::write(
            &declaration,
            "declare module 'local' {\n  interface Context { value: string; }\n}\n",
        )
        .unwrap();

        assert!(!is_suppressed_false_positive(&diagnostic(
            declaration,
            2717,
            "Subsequent property declarations must have the same type.  Property 'value' must be of type 'number', but here has type 'string'.",
        )));
    }
}
