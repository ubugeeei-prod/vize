use super::ConfigDocument;
use crate::config::{LintRuleSeverity, VueVersion};

#[test]
fn document_retains_only_host_observed_dependency_paths() {
    let mut document: ConfigDocument =
        serde_json::from_str(r#"{"configDependencies":["authored.ts"],"compiler":{"vapor":true}}"#)
            .unwrap();
    assert!(document.config_dependencies().is_empty());
    document.set_config_dependencies([
        "/project/vite.config.ts".to_owned(),
        "/project/日本語/🎨.ts".to_owned(),
    ]);
    assert_eq!(
        document.config_dependencies(),
        ["/project/vite.config.ts", "/project/日本語/🎨.ts"]
    );
    assert_eq!(document.compiler_vapor(), Some(true));
}

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
        (
            r#"{"compiler":{"whitespace":"vue2-preserve"}}"#,
            Some("vue2-preserve"),
        ),
    ] {
        let document: ConfigDocument = serde_json::from_str(json).unwrap();
        assert_eq!(document.compiler_whitespace(), expected);
    }
}

#[test]
fn document_projects_compiler_ssr() {
    for (json, expected) in [
        (r#"{}"#, None),
        (r#"{"compiler":{"ssr":true}}"#, Some(true)),
        (r#"{"compiler":{"ssr":false}}"#, Some(false)),
    ] {
        let document: ConfigDocument = serde_json::from_str(json).unwrap();
        assert_eq!(document.compiler_ssr(), expected, "{json}");
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

#[test]
fn document_projects_compiler_entries_in_declaration_order() {
    let document: ConfigDocument = serde_json::from_str(
        r#"{
            "entries": [
                { "basePath": "src", "files": ["legacy/**"],
                  "compiler": { "whitespace": "condense", "vapor": true } },
                { "files": ["modern/**"],
                  "compiler": { "templateSyntax": "strict", "customElements": ["x-*"] } },
                { "files": ["lint-only/**"], "linter": { "enabled": false } }
            ]
        }"#,
    )
    .unwrap();
    let entries = document.compiler_entries();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].base_path.as_deref(), Some("src"));
    assert_eq!(entries[0].files.as_deref().unwrap(), ["legacy/**"]);
    assert_eq!(entries[0].whitespace.as_deref(), Some("condense"));
    assert_eq!(entries[0].vapor, Some(true));
    assert_eq!(entries[1].template_syntax, Some("strict"));
    assert_eq!(entries[1].custom_elements, ["x-*"]);
}

#[test]
fn formatter_preset_assigns_options_that_the_config_does_not_write() {
    let document: ConfigDocument = serde_json::from_str(
        r#"{"formatter":{"preset":"prettier","printWidth":120,"sortBlocks":true,"useTabs":null}}"#,
    )
    .unwrap();
    let profile = document.formatter_profile().clone();
    assert_eq!(
        profile.preset,
        Some(crate::config::FormatterPreset::Prettier)
    );
    assert_eq!(profile.options["printWidth"], 120);
    assert_eq!(profile.options["normalizeCssQuotes"], true);
    assert!(profile.options.get("useTabs").is_none());
    let (config, _) = document.into_config_and_features();
    assert_eq!(config.formatter.print_width, 120);
    assert!(config.formatter.sort_blocks);
    assert!(!config.formatter.sort_attributes);
    assert!(!config.formatter.normalize_directive_shorthands);
}

#[test]
fn formatter_profile_falls_back_to_the_legacy_alias_and_refuses_unknown_presets() {
    let document: ConfigDocument =
        serde_json::from_str(r#"{"fmt":{"preset":"prettier"}}"#).unwrap();
    assert_eq!(document.formatter_profile().options["printWidth"], 80);
    assert_eq!(
        document.into_config_and_features().0.formatter.print_width,
        80
    );

    let unset: ConfigDocument = serde_json::from_str(r#"{"formatter":{}}"#).unwrap();
    assert!(unset.formatter_profile().options.is_null());
    assert!(serde_json::from_str::<ConfigDocument>(r#"{"formatter":{"preset":"x"}}"#).is_err());
}

#[test]
fn compact_formatter_preset_is_typed_and_authored_options_win() {
    let document: ConfigDocument = serde_json::from_str(
        r#"{"formatter":{"preset":"compact","printWidth":108,"bracketSameLine":false}}"#,
    )
    .unwrap();
    let profile = document.formatter_profile().clone();
    assert_eq!(
        profile.preset,
        Some(crate::config::FormatterPreset::Compact)
    );
    assert_eq!(profile.options["printWidth"], 108);
    assert_eq!(profile.options["bracketSameLine"], false);
    assert_eq!(profile.options["sortAttributes"], false);
    assert_eq!(profile.options["sortBlocks"], false);
    assert_eq!(profile.options["normalizeDirectiveShorthands"], true);

    let (config, _) = document.into_config_and_features();
    assert_eq!(config.formatter.print_width, 108);
    assert!(!config.formatter.bracket_same_line);
    assert!(!config.formatter.sort_attributes);
    assert!(!config.formatter.sort_blocks);
}

#[test]
fn documentation_formatter_preset_is_typed_and_authored_options_win() {
    let document: ConfigDocument = serde_json::from_str(
        r#"{"formatter":{"preset":"documentation","printWidth":96,"singleAttributePerLine":false}}"#,
    )
    .unwrap();
    let profile = document.formatter_profile().clone();
    assert_eq!(
        profile.preset,
        Some(crate::config::FormatterPreset::Documentation)
    );
    assert_eq!(profile.options["printWidth"], 96);
    assert_eq!(profile.options["singleAttributePerLine"], false);
    assert_eq!(profile.options["sortAttributes"], false);
    assert_eq!(profile.options["sortBlocks"], false);
    assert_eq!(profile.options["normalizeDirectiveShorthands"], true);

    let (config, _) = document.into_config_and_features();
    assert_eq!(config.formatter.print_width, 96);
    assert!(!config.formatter.single_attribute_per_line);
    assert!(!config.formatter.sort_attributes);
    assert!(!config.formatter.sort_blocks);
}
