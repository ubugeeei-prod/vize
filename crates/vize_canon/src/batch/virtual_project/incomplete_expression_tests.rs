//! An unfinished template expression must not make valid `.vue` imports
//! unresolvable. The editor type-checks one virtual module per SFC; a syntax
//! error in that module is published against every import (#7192).

use std::fs;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use vize_carton::{String as CompactString, ToCompactString};

use super::super::import_rewriter::ImportRewriter;
use super::generate_vue_document_virtual_ts;
use crate::virtual_ts::VirtualTsOptions;

#[test]
fn incomplete_template_expression_keeps_vue_imports_parseable() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("Child.vue"), "<template><span /></template>\n").unwrap();
    let parent = root.join("Parent.vue");
    let script = "<script setup lang=\"ts\">\nimport Child from './Child.vue'\n</script>\n";
    let clean = format!("{script}<template><Child label=\"x\" /></template>\n");
    let cases = [
        (
            "member",
            format!("{script}<template><Child label=\"x\" />{{{{ foo. }}}}</template>\n"),
            true,
        ),
        (
            "prop",
            format!("{script}<template><Child label=\"x\" :title=\"foo.\" /></template>\n"),
            true,
        ),
        (
            "v-if",
            format!("{script}<template><Child v-if=\"foo.\" label=\"x\" /></template>\n"),
            true,
        ),
        (
            "v-for",
            format!(
                "{script}<template><Child v-for=\"item in foo.\" :key=\"item\" label=\"x\" /></template>\n"
            ),
            true,
        ),
        (
            "unclosed",
            format!("{script}<template><Child label=\"x\" />{{{{ foo</template>\n"),
            false,
        ),
    ];

    let rewriter = ImportRewriter::new();
    let clean_code = generate(&parent, &clean, &rewriter);
    let clean_import = import_line(&clean_code);
    assert!(
        clean_import.contains("./Child.vue"),
        "clean import was not kept:\n{clean_code}"
    );
    assert_parses("clean", &clean_code);

    for (name, source, repaired_member) in cases {
        let code = generate(&parent, &source, &rewriter);
        assert_eq!(
            import_line(&code),
            clean_import,
            "{name} changed the .vue import:\n{code}"
        );
        if repaired_member {
            assert!(
                code.contains("foo. x"),
                "{name} dropped the member access:\n{code}"
            );
            assert!(
                ["foo.;", "foo.)", "foo.,"]
                    .iter()
                    .all(|broken| !code.contains(broken)),
                "{name} still emits a broken member tail:\n{code}"
            );
        }
        assert_parses(name, &code);
    }
}

fn generate(path: &std::path::Path, source: &str, rewriter: &ImportRewriter) -> CompactString {
    fs::write(path, source).unwrap();
    generate_vue_document_virtual_ts(path, source, &VirtualTsOptions::default(), rewriter, false)
        .unwrap_or_else(|error| panic!("{error}"))
        .code
}

fn import_line(code: &str) -> CompactString {
    code.lines()
        .find(|line| line.contains("from './Child.vue") || line.contains("from \"./Child.vue"))
        .unwrap_or_default()
        .to_compact_string()
}

fn assert_parses(name: &str, code: &str) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::ts()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "{name} virtual TS did not parse: {:?}\n{code}",
        parsed.diagnostics
    );
}
