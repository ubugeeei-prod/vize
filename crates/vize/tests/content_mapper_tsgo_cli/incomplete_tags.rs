//! Standard tsgo keeps script diagnostics while a template tag is unfinished.

use super::*;
use vize_canon::generate_vue_content_mapper_transform;

const EXPECTATIONS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-incomplete-tag/expectations.json"
));

fn fixture_root() -> PathBuf {
    workspace_root().join("tests/_fixtures/differential/typechecker/content-mapper-incomplete-tag")
}

fn diagnostic_lines(output: &Output) -> Vec<CompactString> {
    assert!(output.stderr.is_empty(), "{}", output_text(output));
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| CompactString::from(line.replace('\\', "/")))
        .collect()
}

fn authored_position(source: &str, byte: usize) -> (usize, usize) {
    assert!(source.is_char_boundary(byte));
    let prefix = &source[..byte];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().encode_utf16().count() + 1;
    (line, column)
}

fn expected_diagnostics(source: &str, script: &CompactString) -> Vec<CompactString> {
    let mapper = generate_vue_content_mapper_transform(Path::new("Recovery.vue"), source).unwrap();
    let mut diagnostics = vec![script.clone()];
    for diagnostic in mapper.diagnostics {
        assert_eq!(diagnostic.code, 100_002);
        assert!(matches!(
            diagnostic.message_text.as_str(),
            "Template parse error: EOF in tag."
                | "Template parse error: Element is missing end tag."
        ));
        let (line, column) = authored_position(source, diagnostic.start);
        diagnostics.push(cstr!(
            "src/Recovery.vue({line},{column}): error TS{}: {}",
            diagnostic.code,
            diagnostic.message_text
        ));
    }
    diagnostics.sort();
    diagnostics
}

#[test]
fn standard_tsgo_preserves_authored_script_errors_through_unfinished_template_edits() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let oracle: serde_json::Value = serde_json::from_str(EXPECTATIONS).unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        let original =
            std::fs::read_to_string(fixture_root().join(case["file"].as_str().unwrap())).unwrap();
        for newline in ["\n", "\r\n"] {
            let cases_root = workspace_root().join("target/vize-tests/tests");
            std::fs::create_dir_all(&cases_root).unwrap();
            let project = tempfile::Builder::new()
                .prefix("content-mapper-incomplete-tag-")
                .tempdir_in(cases_root)
                .unwrap();
            install_mapper_manifest(project.path());
            install_vue_package(project.path());
            std::fs::create_dir(project.path().join("src")).unwrap();
            std::fs::write(
                project.path().join("tsconfig.json"),
                serde_json::to_vec_pretty(&json!({
                    "compilerOptions": {
                        "strict": true, "skipLibCheck": true, "target": "ES2022",
                        "module": "ESNext", "moduleResolution": "bundler", "jsx": "preserve"
                    },
                    "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
                    "files": ["src/Recovery.vue"]
                }))
                .unwrap(),
            )
            .unwrap();
            let complete = original.replace('\n', newline);
            let template = complete.find("<template>").unwrap();
            let unfinished = cstr!(
                "{}<template><Loc</template>{newline}",
                &complete[..template]
            );
            let script_diagnostic = cstr!(
                "src/Recovery.vue({},{}): error TS2322: {}",
                case["line"].as_u64().unwrap(),
                case["column"].as_u64().unwrap(),
                oracle["scriptMessage"].as_str().unwrap()
            );
            for (state, source) in [
                ("complete", complete.as_str()),
                ("unfinished-tag", unfinished.as_str()),
                ("template-repaired", complete.as_str()),
            ] {
                std::fs::write(project.path().join("src/Recovery.vue"), source).unwrap();
                let output = check_project(&tsgo, project.path(), "tsconfig.json");
                assert!(
                    !output.status.success(),
                    "{state}: {}",
                    output_text(&output)
                );
                let mut actual = diagnostic_lines(&output);
                actual.sort();
                let expected = expected_diagnostics(source, &script_diagnostic);
                assert_eq!(actual, expected, "{state}: {}", output_text(&output));
                if state == "unfinished-tag" {
                    assert!(
                        expected.len() > 1,
                        "unfinished tag must still report parser errors"
                    );
                } else {
                    assert_eq!(expected, vec![script_diagnostic.clone()]);
                }
            }
            let repaired = complete.replace("'broken'", "1");
            std::fs::write(project.path().join("src/Recovery.vue"), repaired).unwrap();
            let output = check_project(&tsgo, project.path(), "tsconfig.json");
            assert_success(&output);
            assert!(
                diagnostic_lines(&output).is_empty(),
                "{}",
                output_text(&output)
            );
        }
    }
}
