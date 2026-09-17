#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_dom::DomCompilerOptions;
use vize_atelier_sfc::{SfcCompileOptions, compile_sfc, parse_sfc};

const ARMS: &str =
    r#"<p v-when="{ kind: 'ok', const data }">{{ data }}</p><i v-when="_">other</i>"#;

fn options(backend: &str) -> SfcCompileOptions {
    let mut options = SfcCompileOptions {
        vapor: backend == "vapor",
        ..Default::default()
    };
    options.template.ssr = backend == "ssr";
    options.template.compiler_options = Some(DomCompilerOptions {
        experimental_patterned_template: true,
        ..Default::default()
    });
    options
}

fn compile(source: &str, backend: &str) -> String {
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let result = compile_sfc(&descriptor, options(backend)).unwrap();
    assert!(result.errors.is_empty(), "{backend}: {:?}", result.errors);
    result.code.to_string()
}

#[test]
fn root_match_is_equivalent_to_nested_match_in_every_sfc_lane() {
    for backend in ["vdom", "ssr", "vapor"] {
        for script in [
            "",
            "<script>export default { props: ['result'] }</script>",
            "<script setup>const result = { kind: 'ok', data: 'ready' }</script>",
        ] {
            let root = format!("{script}<template v-match=\"result\">{ARMS}</template>");
            let nested = format!(
                "{script}<template><template v-match=\"result\">{ARMS}</template></template>"
            );
            assert_eq!(
                compile(&root, backend),
                compile(&nested, backend),
                "{backend}, {script}"
            );
            let descriptor = parse_sfc(&root, Default::default()).unwrap();
            assert_eq!(descriptor.template.as_ref().unwrap().content, ARMS);
        }
    }
}

#[test]
fn root_subject_changes_invalidate_template_hash_without_changing_body() {
    let first = parse_sfc(
        "<template v-match=\"first\"><p v-when=\"_\"/></template>",
        Default::default(),
    )
    .unwrap();
    let second = parse_sfc(
        "<template v-match=\"second\"><p v-when=\"_\"/></template>",
        Default::default(),
    )
    .unwrap();
    assert_eq!(
        first.template.as_ref().unwrap().content,
        second.template.as_ref().unwrap().content
    );
    assert_ne!(first.template_hash(), second.template_hash());
}

#[test]
fn header_metadata_does_not_become_runtime_directives_or_move_authored_content() {
    for backend in ["vdom", "ssr", "vapor"] {
        let root = format!(
            "<!-- \u{1f600} -->\r\n<template lang=\"html\" data-note=\"\u{e9}\"\r\n v-if=\"false\" v-match=\"result\">{ARMS}</template>"
        );
        let nested = format!("<template><template v-match=\"result\">{ARMS}</template></template>");
        assert_eq!(compile(&root, backend), compile(&nested, backend));
    }
}

#[test]
fn root_subject_keeps_imports_used_only_in_the_header() {
    for backend in ["vdom", "ssr", "vapor"] {
        let script = "<script setup>import { result } from './state'</script>";
        let root = format!("{script}<template v-match=\"result\">{ARMS}</template>");
        let nested =
            format!("{script}<template><template v-match=\"result\">{ARMS}</template></template>");
        let output = compile(&root, backend);
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &output, oxc_span::SourceType::mjs()).parse();
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let bindings: Vec<_> = parsed
            .program
            .body
            .iter()
            .filter_map(|statement| {
                let oxc_ast::ast::Statement::ImportDeclaration(import) = statement else {
                    return None;
                };
                (import.source.value == "./state").then(|| {
                    import
                        .specifiers
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|specifier| specifier.local().name.as_str())
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        assert_eq!(bindings, vec![vec!["result"]]);
        assert_eq!(output, compile(&nested, backend));
    }
}

#[test]
fn adapter_and_repeated_compilation_preserve_the_original_descriptor() {
    use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
    use vize_atelier_sfc::{SfcScriptOutputMode, compile_sfc_for_adapter};
    let source = format!(
        "<script setup>import {{ result }} from './state'</script><template v-match=\"result\">{ARMS}</template>"
    );
    let nested = source.replace("<template v-match", "<template><template v-match") + "</template>";
    let descriptor = parse_sfc(&source, Default::default()).unwrap();
    let inner = parse_sfc(&nested, Default::default()).unwrap();
    let original = serde_json::to_value(&descriptor).unwrap();
    for backend in ["vdom", "ssr", "vapor"] {
        let compile = |descriptor| {
            compile_sfc_for_adapter(
                descriptor,
                options(backend),
                TemplateSyntaxMode::Standard,
                CustomElementMatcher::default(),
                CodegenOptions::default(),
                SfcScriptOutputMode::SeparateTemplate,
            )
            .unwrap()
        };
        let expected = compile(&inner);
        assert!(expected.errors.is_empty());
        for _ in 0..2 {
            let actual = compile(&descriptor);
            assert!(actual.errors.is_empty());
            assert_eq!(actual.code, expected.code);
            assert_eq!(serde_json::to_value(&descriptor).unwrap(), original);
        }
    }
}

#[test]
fn invalid_and_disabled_headers_report_the_authored_directive() {
    for backend in ["vdom", "ssr", "vapor"] {
        for directive in [
            "v-match",
            "v-match=\"\"",
            "v-match:arg=\"result\"",
            "v-match.foo=\"result\"",
        ] {
            let source = format!(
                "<!-- \u{1f600} -->\r\n<template lang=\"html\"\r\n {directive}><p v-when=\"_\"/></template>"
            );
            let descriptor = parse_sfc(&source, Default::default()).unwrap();
            let error = compile_sfc(&descriptor, options(backend)).unwrap_err();
            assert_eq!(error.code.as_deref(), Some("V_MATCH_SYNTAX"));
            let loc = error.loc.unwrap();
            assert_eq!(&source[loc.start..loc.end], directive);
            assert_eq!((loc.start_line, loc.start_column), (3, 2));
        }
        let source = "<template v-match=\"result\"><p v-when=\"_\"/></template>";
        let descriptor = parse_sfc(source, Default::default()).unwrap();
        let mut disabled = options(backend);
        disabled
            .template
            .compiler_options
            .as_mut()
            .unwrap()
            .experimental_patterned_template = false;
        let error = compile_sfc(&descriptor, disabled).unwrap_err();
        assert_eq!(
            error.message,
            "`v-match` / `v-when` patterned templates require `experimentals.patternedTemplate`."
        );
    }
}

#[test]
fn transformed_or_missing_source_metadata_is_rejected_without_a_panic() {
    let source = "<template v-match=\"result\"><p v-when=\"_\"/></template>";
    for mutation in 0..3 {
        let mut descriptor = parse_sfc(source, Default::default()).unwrap();
        let template = descriptor.template.as_mut().unwrap();
        match mutation {
            0 => template.content = "<span/>".into(),
            1 => template.loc.tag_start = usize::MAX,
            _ => template.loc.start = usize::MAX,
        }
        assert_eq!(
            compile_sfc(&descriptor, options("vdom"))
                .unwrap_err()
                .message,
            "Root v-match requires preserved SFC source metadata."
        );
    }
    for metadata in ["src=\"./view.html\"", "lang=\"pug\""] {
        let source =
            format!("<template {metadata} v-match=\"result\"><p v-when=\"_\"/></template>");
        let descriptor = parse_sfc(&source, Default::default()).unwrap();
        assert_eq!(
            compile_sfc(&descriptor, options("vdom"))
                .unwrap_err()
                .message,
            "Root v-match requires an inline HTML template."
        );
    }
}

#[test]
fn malformed_header_metadata_is_not_hidden_by_masking() {
    let source =
        "<!-- \u{1f600} -->\r\n<template v-match=\"result\"\r\n foo=><p v-when=\"_\"/></template>";
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    for backend in ["vdom", "ssr", "vapor"] {
        let error = compile_sfc(&descriptor, options(backend)).unwrap_err();
        assert_eq!(error.code.as_deref(), Some("TEMPLATE_ERROR"));
        assert_eq!(
            error.message,
            "Attribute `foo` is missing a value after `=`; continuing without the value."
        );
        let loc = error.loc.unwrap();
        assert_eq!(loc.start, source.find("foo=>").unwrap() + 4);
        assert_eq!(loc.end, loc.start + 1);
        assert_eq!((loc.start_line, loc.start_column), (3, 6));
        assert_eq!(&source[loc.start..loc.end], ">");

        let recovered =
            format!("<template v-match=\"result\" foo=\"a\" foo=\"b\">{ARMS}</template>");
        let clean = format!("<template v-match=\"result\">{ARMS}</template>");
        assert_eq!(compile(&recovered, backend), compile(&clean, backend));
    }
}

#[test]
fn header_validation_respects_parser_options() {
    use vize_atelier_core::TemplateSyntaxMode;
    use vize_atelier_sfc::compile_sfc_with_template_syntax;
    for backend in ["vdom", "ssr", "vapor"] {
        let clean = format!("<template v-match=\"result\">{ARMS}</template>");
        let quirky = format!("<template v-match=\"result\"foo=\"ok\">{ARMS}</template>");
        let descriptor = parse_sfc(&quirky, Default::default()).unwrap();
        let result = compile_sfc_with_template_syntax(
            &descriptor,
            options(backend),
            TemplateSyntaxMode::Quirks,
        )
        .unwrap();
        assert_eq!(result.code, compile(&clean, backend));
        assert_eq!(
            compile_sfc(&descriptor, options(backend))
                .unwrap_err()
                .code
                .as_deref(),
            Some("TEMPLATE_ERROR")
        );

        let commented = r#"<template v-match="result"><p v-when="{ kind: 'ok', const data }"
 // body metadata
>{{ data }}</p><i v-when="_">other</i></template>"#;
        let descriptor = parse_sfc(commented, Default::default()).unwrap();
        let mut enabled = options(backend);
        enabled
            .template
            .compiler_options
            .as_mut()
            .unwrap()
            .experimental_in_tag_comments = true;
        let result = compile_sfc(&descriptor, enabled).unwrap();
        assert_eq!(result.code, compile(&clean, backend));
    }
}

#[test]
fn root_match_renders_updates_and_evaluates_the_subject_once() {
    use serde_json::{Value, json};
    use std::{
        io::Write,
        path::Path,
        process::{Command, Stdio},
    };
    let paragraph = |text| json!([{"tag": "p", "attributes": {}, "children": [text]}]);
    for backend in ["vdom", "vapor", "ssr"] {
        let source = r#"<template v-match="readSubject()"><p v-when="'ready'">Ready</p><p v-when="_">Other</p></template>"#;
        let code = compile(source, backend);
        let mut child = Command::new("node")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/tooling/support/patterned-template-runtime.mjs"),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let input = json!({"backend": backend, "code": code, "cases": [{
            "scenario": "subject-evaluation", "context": {"subject": "ready"},
            "steps": [{"patch": {"subject": "waiting"}}, {"patch": {"subject": "ready"}}],
            "trees": [paragraph("Ready"), paragraph("Other"), paragraph("Ready")], "reads": 3
        }]});
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.to_string().as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{backend}: {}\n{code}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            json!({"passed": 1})
        );
    }
}
