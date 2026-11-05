use crate::{
    LintDiagnostic, LintPreset, LintResult, Linter, OutputFormat, format_results, rule_docs_path,
};
use vize_l0::ToCompactString;

#[test]
fn json_output_uses_source_line_columns() {
    let source = r#"<script setup lang="ts">
const items = [1]
</script>

<template>
  <div v-for="item in items">{{ item }}</div>
</template>
"#;
    let filename = vize_l0::String::from("Component.vue");
    let result = Linter::new().lint_sfc(source, &filename);
    let output = format_results(
        &[result],
        &[(filename, vize_l0::String::from(source))],
        OutputFormat::Json,
    );

    assert!(output.contains(r#""line": 6"#), "{output}");
    assert!(output.contains(r#""column": 3"#), "{output}");
    assert!(
        output
            .contains(r#""ruleDocsPath": "docs/content/rules/reference/vue-require-v-for-key.md""#),
        "{output}"
    );
}

#[test]
fn json_output_uses_character_columns_after_multibyte_text() {
    let source = r#"<template><div title="café" v-html="x"></div></template>"#;
    let filename = vize_l0::String::from("Component.vue");
    let start = source.find("v-html").unwrap() as u32;
    let end = start + "v-html".len() as u32;
    let result = LintResult {
        filename: filename.clone(),
        diagnostics: vec![LintDiagnostic::warn(
            "vue/no-v-html",
            "Avoid raw HTML",
            start,
            end,
        )],
        error_count: 0,
        warning_count: 1,
    };
    let output = format_results(
        &[result],
        &[(filename, vize_l0::String::from(source))],
        OutputFormat::Json,
    );
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let diagnostic = &json[0]["messages"][0];

    assert_eq!(diagnostic["line"], 1);
    assert_eq!(diagnostic["column"], 29);
    assert_eq!(diagnostic["endLine"], 1);
    assert_eq!(diagnostic["endColumn"], 35);
}

#[test]
fn json_output_includes_template_parser_diagnostic_locations() {
    let source = r#"<script setup lang="ts">
const msg = "hello";
</script>

<template>
  <div>
    <span>{{ msg }}
  </div>
</template>
"#;
    let filename = vize_l0::String::from("Component.vue");
    let result = Linter::new().lint_sfc(source, &filename);
    let output = format_results(
        &[result],
        &[(filename, vize_l0::String::from(source))],
        OutputFormat::Json,
    );
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    let messages = json[0]["messages"].as_array().unwrap();
    let diagnostic = messages
        .iter()
        .find(|message| message["ruleId"] == "parser/template")
        .expect("parser/template diagnostic should be present");

    assert_eq!(diagnostic["severity"], 2);
    assert_eq!(diagnostic["line"], 7);
    assert!(diagnostic["column"].as_u64().unwrap() > 1);
    assert!(
        diagnostic["endLine"].as_u64().unwrap() >= diagnostic["line"].as_u64().unwrap(),
        "{output}"
    );
    assert!(
        diagnostic["endColumn"].as_u64().unwrap() > diagnostic["column"].as_u64().unwrap(),
        "{output}"
    );
}

#[test]
fn output_format_parses_report_formats() {
    assert_eq!(OutputFormat::parse("stylish"), Some(OutputFormat::Stylish));
    assert_eq!(OutputFormat::parse("ansi"), Some(OutputFormat::Ansi));
    assert_eq!(OutputFormat::parse("anssi"), Some(OutputFormat::Ansi));
    assert_eq!(OutputFormat::parse("plain-text"), Some(OutputFormat::Plain));
    assert_eq!(OutputFormat::parse("md"), Some(OutputFormat::Markdown));
    assert_eq!(OutputFormat::parse("telegraph"), Some(OutputFormat::Agent));
    assert_eq!(OutputFormat::parse("unknown"), None);
}

#[test]
fn rendered_outputs_interpolate_lint_message_placeholders() {
    let source = r#"<script setup lang="ts">
defineOptions({
  name: "Label",
})

const url = ""
</script>

<template>
  <a :href="url" />
</template>
"#;
    let filename = vize_l0::String::from("Label.vue");
    let result = Linter::with_preset(LintPreset::Essential).lint_sfc(source, &filename);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Component name \"Label\"")),
        "{:?}",
        result.diagnostics
    );
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Dynamic :href binding")),
        "{:?}",
        result.diagnostics
    );

    let sources = [(filename.clone(), vize_l0::String::from(source))];
    for format in [
        OutputFormat::Text,
        OutputFormat::Ansi,
        OutputFormat::Plain,
        OutputFormat::Stylish,
        OutputFormat::Json,
        OutputFormat::Markdown,
        OutputFormat::Html,
        OutputFormat::Agent,
    ] {
        let output = format_results(std::slice::from_ref(&result), &sources, format);
        assert!(!output.contains("{name}"), "{format:?}\n{output}");
        assert!(!output.contains("{attr}"), "{format:?}\n{output}");
        assert!(output.contains("Label"), "{format:?}\n{output}");
        assert!(output.contains("href"), "{format:?}\n{output}");
    }
}

#[test]
fn rule_docs_path_maps_namespaces_to_reference_pages() {
    assert_eq!(
        rule_docs_path("vue/require-v-for-key"),
        "docs/content/rules/reference/vue-require-v-for-key.md"
    );
    assert_eq!(
        rule_docs_path("a11y/img-alt"),
        "docs/content/rules/reference/a11y-img-alt.md"
    );
    assert_eq!(
        rule_docs_path("script/no-options-api"),
        "docs/content/rules/reference/script-no-options-api.md"
    );
    assert_eq!(
        rule_docs_path("nuxt/prefer-import-meta"),
        "docs/content/rules/reference/nuxt-prefer-import-meta.md"
    );
    assert_eq!(
        rule_docs_path("petite-vue/valid-v-scope"),
        "docs/content/rules/reference/petite-vue-valid-v-scope.md"
    );
    assert_eq!(
        rule_docs_path("vize:croquis/cf/array-mutation"),
        "docs/content/rules/project/vize-croquis-cf-array-mutation.md"
    );
    assert_eq!(
        rule_docs_path("vue/cross-file-attrs-fallthrough"),
        "docs/content/rules/project/vue-cross-file-attrs-fallthrough.md"
    );
}

#[test]
fn stylish_output_includes_reference_paths() {
    let source = "<template><div v-for=\"item in items\"></div></template>";
    let filename = vize_l0::String::from("Component.vue");
    let result = Linter::new().lint_sfc(source, &filename);
    let output = format_results(
        &[result],
        &[(filename, vize_l0::String::from(source))],
        OutputFormat::Stylish,
    );

    assert!(output.contains("Component.vue"), "{output}");
    assert!(output.contains("vue/require-v-for-key"), "{output}");
    assert!(
        output.contains("docs/content/rules/reference/vue-require-v-for-key.md"),
        "{output}"
    );
}

#[test]
fn text_output_includes_reference_paths() {
    let result = LintResult {
        filename: "Component.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::warn(
            "vue/no-v-html",
            "Avoid raw HTML",
            0,
            3,
        )],
        error_count: 0,
        warning_count: 1,
    };
    let output = format_results(
        &[result],
        &[(
            vize_l0::String::from("Component.vue"),
            vize_l0::String::from("abc"),
        )],
        OutputFormat::Text,
    );

    assert!(
        output.contains("docs/content/rules/reference/vue-no-v-html.md"),
        "{output}"
    );
}

#[test]
fn ansi_output_includes_summary_and_ansi_help() {
    let result = LintResult {
        filename: "Component.vue".to_compact_string(),
        diagnostics: vec![
            LintDiagnostic::warn("vue/no-v-html", "Avoid raw HTML", 0, 3)
                .with_help("Use **text interpolation** instead."),
        ],
        error_count: 0,
        warning_count: 1,
    };
    let output = format_results(
        &[result],
        &[(
            vize_l0::String::from("Component.vue"),
            vize_l0::String::from("abc"),
        )],
        OutputFormat::Ansi,
    );

    assert!(
        output.contains("docs/content/rules/reference/vue-no-v-html.md"),
        "{output}"
    );
    assert!(output.contains("1 warning in 1 file"), "{output}");
    assert!(output.contains("\x1b["), "{output}");
}

#[test]
fn plain_output_includes_reference_paths_without_ansi() {
    let result = LintResult {
        filename: "Component.vue".to_compact_string(),
        diagnostics: vec![
            LintDiagnostic::error("a11y/img-alt", "Missing alt text", 0, 3)
                .with_help("Add an `alt` attribute."),
        ],
        error_count: 1,
        warning_count: 0,
    };
    let output = format_results(
        &[result],
        &[(
            vize_l0::String::from("Component.vue"),
            vize_l0::String::from("abc"),
        )],
        OutputFormat::Plain,
    );

    assert!(
        output.contains("Patina lint report: 1 error in 1 file"),
        "{output}"
    );
    assert!(
        output.contains("docs/content/rules/reference/a11y-img-alt.md"),
        "{output}"
    );
    assert!(output.contains("Add an alt attribute."), "{output}");
    assert!(!output.contains("\x1b["), "{output}");
}

#[test]
fn markdown_output_keeps_help_and_reference_paths() {
    let result = LintResult {
        filename: "Component.vue".to_compact_string(),
        diagnostics: vec![
            LintDiagnostic::warn("vue/no-v-html", "Avoid raw HTML", 10, 16)
                .with_help("Use text interpolation instead."),
        ],
        error_count: 0,
        warning_count: 1,
    };
    let output = format_results(
        &[result],
        &[(
            vize_l0::String::from("Component.vue"),
            vize_l0::String::from(""),
        )],
        OutputFormat::Markdown,
    );

    assert!(output.contains("# Patina Lint Report"), "{output}");
    assert!(
        output.contains("Reference: `docs/content/rules/reference/vue-no-v-html.md`"),
        "{output}"
    );
    assert!(
        output.contains("Use text interpolation instead."),
        "{output}"
    );
}

#[test]
fn html_output_escapes_messages() {
    let result = LintResult {
        filename: "Component.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::error(
            "html/deprecated-element",
            "Avoid <center> & friends",
            0,
            8,
        )],
        error_count: 1,
        warning_count: 0,
    };
    let output = format_results(
        &[result],
        &[(
            vize_l0::String::from("Component.vue"),
            vize_l0::String::from("<center>"),
        )],
        OutputFormat::Html,
    );

    assert!(output.contains("&lt;center&gt; &amp; friends"), "{output}");
    assert!(
        output.contains("docs/content/rules/reference/html-deprecated-element.md"),
        "{output}"
    );
}

#[test]
fn agent_output_is_line_oriented() {
    let result = LintResult {
        filename: "Component.vue".to_compact_string(),
        diagnostics: vec![LintDiagnostic::warn(
            "ssr/no-hydration-mismatch",
            "SSR risk",
            0,
            3,
        )],
        error_count: 0,
        warning_count: 1,
    };
    let output = format_results(
        &[result],
        &[(
            vize_l0::String::from("Component.vue"),
            vize_l0::String::from("abc"),
        )],
        OutputFormat::Agent,
    );

    assert!(
        output.starts_with("patina report errors=0 warnings=1 files=1"),
        "{output}"
    );
    assert!(
        output.contains("docs=\"docs/content/rules/reference/ssr-no-hydration-mismatch.md\""),
        "{output}"
    );
    assert!(output.contains("message: SSR risk"), "{output}");
}

/// Rule ids and catalogue links from `docs/content/rules/all.md`.
fn published_rule_pages(catalogue: &str) -> Vec<(&str, &str)> {
    let mut pages = Vec::new();
    let mut rest = catalogue;
    while let Some(start) = rest.find("[`") {
        rest = &rest[start + 2..];
        let Some(id_end) = rest.find("`](./") else {
            break;
        };
        let id = &rest[..id_end];
        rest = &rest[id_end + 5..];
        let Some(path_end) = rest.find(')') else {
            break;
        };
        let relative = &rest[..path_end];
        rest = &rest[path_end + 1..];
        if relative.starts_with("reference/") || relative.starts_with("project/") {
            pages.push((id, relative));
        }
    }
    pages
}

#[test]
fn rule_docs_path_exists_for_every_published_rule() {
    const CATALOGUE: &str = include_str!("../../../../docs/content/rules/all.md");
    let pages = published_rule_pages(CATALOGUE);
    let reference = pages
        .iter()
        .filter(|(_, relative)| relative.starts_with("reference/"))
        .count();
    let project = pages.len() - reference;
    assert!(
        reference >= 200 && project >= 60,
        "published catalogue was not read: reference={reference} project={project}"
    );

    let rules_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/content/rules");
    let mut missing = vize_l0::String::new("");
    for (id, relative) in pages {
        let path = rule_docs_path(id);
        // This operational check retains its established guide authority. The
        // generated rule page supplies examples and links back to that guide.
        let expected = if id == "vue/comment-directive" {
            vize_l0::String::from("docs/content/guide/lint-execution.md")
        } else {
            vize_l0::cstr!("docs/content/rules/{relative}")
        };
        assert_eq!(path, expected, "{id}");
        if !rules_dir.join(relative).is_file() {
            if !missing.is_empty() {
                missing.push('\n');
            }
            vize_l0::append!(missing, "{id} -> {path}");
        }
    }
    assert!(
        missing.is_empty(),
        "published rules with no page:\n{missing}"
    );
}

#[test]
fn synthetic_diagnostic_routes_keep_their_owned_documentation() {
    const ROUTES: &str = include_str!(
        "../../../../tests/_fixtures/differential/linter/diagnostic-documentation-11818/routes.json"
    );
    const CONTROL: &str = include_str!(
        "../../../../tests/_fixtures/differential/linter/diagnostic-documentation-11818/DocumentationControl.vue.txt"
    );
    let input: serde_json::Value = serde_json::from_str(ROUTES).unwrap();
    assert_eq!(
        input["schema"],
        "vize.synthetic-diagnostic-documentation.v1"
    );
    assert_eq!(input["version"], 1);
    let routes = input["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 2);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for route in routes {
        let id = route["id"].as_str().unwrap();
        let page = route["page"].as_str().unwrap();
        let expected = vize_l0::cstr!("docs/content/rules/{page}");
        assert_eq!(rule_docs_path(id), expected, "{id}");
        for locale in ["", "ja/"] {
            let path = vize_l0::cstr!("docs/content/{locale}rules/{page}");
            assert!(root.join(path.as_str()).is_file(), "{id}: {path}");
        }
    }
    assert_eq!(
        rule_docs_path(input["unknown"]["id"].as_str().unwrap()).as_str(),
        input["unknown"]["path"].as_str().unwrap()
    );
    let filename = vize_l0::String::from("DocumentationControl.vue");
    let actual = format_results(
        &[Linter::new().lint_sfc(CONTROL, &filename)],
        &[(filename, CONTROL.into())],
        OutputFormat::Json,
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&actual).unwrap(),
        serde_json::json!([{"file":"DocumentationControl.vue","messages":[],"errorCount":0,"warningCount":0}])
    );
}
