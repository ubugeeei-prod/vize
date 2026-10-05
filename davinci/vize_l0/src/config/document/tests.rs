use super::ConfigDocument;
use crate::config::{LintRuleSeverity, VueVersion};

#[test]
fn document_projects_only_documented_compiler_whitespace_modes() {
    for (json, expected) in [
        (r#"{}"#, None),
        (r#"{"compiler":{"whitespace":null}}"#, None),
        (r#"{"compiler":{"whitespace":false}}"#, None),
        (r#"{"compiler":{"whitespace":42}}"#, None),
        (r#"{"compiler":{"whitespace":{}}}"#, None),
        (r#"{"compiler":{"whitespace":[]}}"#, None),
        (r#"{"compiler":{"whitespace":"unknown"}}"#, None),
        (
            r#"{"compiler":{"whitespace":"condense"}}"#,
            Some("condense"),
        ),
        (
            r#"{"compiler":{"whitespace":"preserve"}}"#,
            Some("preserve"),
        ),
        (
            r#"{"compiler":{"whitespace":"\u0070reserve"}}"#,
            Some("preserve"),
        ),
        (
            r#"{"compiler":{"whitespace":"vue2-line-breaks"}}"#,
            Some("vue2-line-breaks"),
        ),
    ] {
        let document: ConfigDocument = serde_json::from_str(json).unwrap();
        assert_eq!(document.compiler_whitespace(), expected);
    }
}

#[test]
fn document_ignores_non_string_whitespace_without_losing_other_compiler_fields() {
    for value in ["1e400", "-1e400", "[1e400]", "{\"nested\":[1e400]}"] {
        let json = format!(r#"{{"compiler":{{"whitespace":{value},"vapor":true}}}}"#);
        let document: ConfigDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(document.compiler_whitespace(), None, "{json}");
        assert_eq!(document.compiler_vapor(), Some(true), "{json}");
    }
    let document: ConfigDocument = serde_json::from_value(serde_json::json!({
        "compiler": { "whitespace": "preserve", "vapor": true }
    }))
    .unwrap();
    assert_eq!(document.compiler_whitespace(), Some("preserve"));
    assert_eq!(document.compiler_vapor(), Some(true));
    for malformed in ["1e", "[1e400,]", "{\"nested\":}"] {
        let json = format!(r#"{{"compiler":{{"whitespace":{malformed}}}}}"#);
        assert!(
            serde_json::from_str::<ConfigDocument>(&json).is_err(),
            "{json}"
        );
    }
}

#[test]
fn document_projects_aliases_and_editor_flags_without_host_state() {
    let document: ConfigDocument = serde_json::from_slice(
        br#"{
            "check": { "globals": "ambient.d.ts", "servers": 2 },
            "fmt": { "singleQuote": true },
            "lsp": { "hover": false, "signatureHelp": false },
            "vue": { "version": "2.7" },
            "typeChecker": { "lspRequestTimeoutMs": 0 }
        }"#,
    )
    .unwrap();
    assert_eq!(document.lsp_request_timeout_ms(), 1);
    assert_eq!(
        document.language_server_unstable_flags().signature_help,
        Some(false)
    );
    let (config, features) = document.into_config_and_features();
    assert_eq!(
        config.type_checker.globals_file.as_deref(),
        Some("ambient.d.ts")
    );
    assert_eq!(config.type_checker.servers, Some(2));
    assert!(config.formatter.single_quote);
    assert_eq!(config.language_server.hover, Some(false));
    assert_eq!(features.vue_version, Some(VueVersion::V2_7));
    assert!(features.type_checker_legacy_vue2);
}

#[test]
fn document_retains_declaration_order_and_entry_base_paths() {
    let document: ConfigDocument = serde_json::from_str(
        r#"{
            "basePath": "日本語/🎨", "files": ["root.vue"],
            "ignores": ["root-ignore.vue"],
            "entries": [
                { "basePath": "first", "files": ["first.vue"],
                  "ignores": ["first-ignore.vue"],
                  "linter": { "rules": { "vue/no-v-html": "warn" } } },
                { "basePath": "second", "files": ["second.vue"],
                  "linter": { "rules": { "vue/no-v-html": "error" } } }
            ]
        }"#,
    )
    .unwrap();
    let ignores = document.entry_ignores();
    assert_eq!(ignores[0].base_path, None);
    assert_eq!(ignores[1].base_path.as_deref(), Some("first"));
    let plan = document.linter_plan();
    assert_eq!(plan.entries[0].base_path.as_deref(), Some("first"));
    assert_eq!(plan.entries[1].base_path.as_deref(), Some("second"));
    assert_eq!(
        plan.entries[0].rules["vue/no-v-html"],
        LintRuleSeverity::Warn
    );
    assert_eq!(
        plan.entries[1].rules["vue/no-v-html"],
        LintRuleSeverity::Error
    );
    let entries = document.into_entry_files();
    assert_eq!(entries[0].base_path.as_deref(), Some("日本語/🎨"));
    assert_eq!(entries[1].files[0], "first.vue");
    assert_eq!(entries[2].files[0], "second.vue");
}
