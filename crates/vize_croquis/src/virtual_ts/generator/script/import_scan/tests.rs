use super::super::import_scan::{
    String, code_line_starts, find_relative_import_from_range, import_statement_end,
    join_statement_lines,
};

fn relative_path(line: &str) -> Option<&str> {
    find_relative_import_from_range(line).map(|(_, start, end, _)| &line[start..end])
}

fn statement_end(content: &str) -> Option<usize> {
    statement_end_at(content, 0)
}

fn statement_end_at(content: &str, start: usize) -> Option<usize> {
    let lines: Vec<&str> = content.lines().collect();
    let code_starts = code_line_starts(&lines);
    import_statement_end(&lines, &code_starts, start)
}

/// The module-scope text emitted for the statement starting at line 0.
fn joined_statement(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let end = statement_end(content).expect("expected an import statement");
    join_statement_lines(&lines[0..=end])
}

#[test]
fn groups_wrapped_named_imports_into_one_statement() {
    let content = "import {\n  createScope,\n  provideScope,\n} from \"./scope.ts\";\nconst a = 1;";
    assert_eq!(statement_end(content), Some(3));
    assert_eq!(
        joined_statement(content),
        "import {\n  createScope,\n  provideScope,\n} from \"./scope.ts\";"
    );
}

#[test]
fn groups_a_from_clause_on_its_own_line() {
    let content = "import {\n  createScope,\n}\nfrom \"./scope.ts\";\nconst a = 1;";
    assert_eq!(statement_end(content), Some(3));
    assert!(joined_statement(content).ends_with("from \"./scope.ts\";"));
}

#[test]
fn groups_a_wrap_after_the_default_binding() {
    let content = "import defaultA,\n  { b } from './x';\nconst a = 1;";
    assert_eq!(statement_end(content), Some(1));
}

#[test]
fn groups_a_bare_from_keyword_line() {
    let content = "import {\n  a,\n}\nfrom\n  './x';\nconst a = 1;";
    assert_eq!(statement_end(content), Some(4));
}

#[test]
fn stops_at_code_trailing_a_complete_import() {
    // The trailing `{` must not drag the function body into the import.
    let content = "import { a } from './x'; function f() {\n  return 1;\n}\nconst b = 2;";
    assert_eq!(statement_end(content), Some(0));
}

#[test]
fn groups_wrapped_import_attributes() {
    let content = "import data from \"./d.json\" with {\n  type: \"json\",\n};\nconst a = 1;";
    assert_eq!(statement_end(content), Some(2));
}

#[test]
fn keeps_line_comments_from_swallowing_later_bindings() {
    let content =
        "import {\n  createScope, // scope factory\n  provideScope,\n} from \"./scope.ts\";";
    assert_eq!(statement_end(content), Some(3));

    // Collapsing the lines would move `// scope factory` in front of the
    // remaining bindings and comment out the rest of the statement.
    let statement = joined_statement(content);
    assert!(statement.contains("\n  provideScope,"));
    assert!(statement.ends_with("from \"./scope.ts\";"));
}

#[test]
fn ignores_braces_and_quotes_inside_comments() {
    let content = "import {\n  createScope, /* } it's fine */\n  provideScope,\n} from \"./scope.ts\";\nconst a = 1;";
    assert_eq!(statement_end(content), Some(3));
}

#[test]
fn ignores_import_like_lines_inside_template_literals() {
    let content = "const sample = `\nimport { ref } from 'vue';\n`;\nconst a = 1;";
    assert_eq!(statement_end_at(content, 1), None);
}

#[test]
fn ignores_import_like_lines_inside_block_comments() {
    let content = "/*\nimport { ref } from 'vue';\n*/\nconst a = 1;";
    assert_eq!(statement_end_at(content, 1), None);
}

#[test]
fn resumes_after_a_template_literal_closes() {
    let content =
        "const sample = `\nimport { ref } from 'vue';\n`;\nimport { computed } from 'vue';";
    assert_eq!(statement_end_at(content, 1), None);
    assert_eq!(statement_end_at(content, 3), Some(3));
}

#[test]
fn tracks_nested_template_expressions() {
    let lines: Vec<&str> = vec![
        "const a = `${ `${ inner }` }`;",
        "import { ref } from 'vue';",
    ];
    assert_eq!(code_line_starts(&lines), vec![true, true]);
}

#[test]
fn keeps_single_line_imports_untouched() {
    let content = "import { ref } from 'vue';\nconst a = 1;";
    assert_eq!(statement_end(content), Some(0));

    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(
        join_statement_lines(&lines[0..=0]),
        "import { ref } from 'vue';"
    );
}

#[test]
fn ignores_lines_that_do_not_start_an_import() {
    assert_eq!(
        statement_end("const a = 1;\nimport { ref } from 'vue';"),
        None
    );
}

#[test]
fn never_consumes_beyond_an_unclosed_import_like_line() {
    // A stray `import`-looking line must not swallow the rest of the body.
    let content = "import foo\nconst a = 1;\nconst b = 2;";
    assert_eq!(statement_end(content), Some(0));
}

#[test]
fn finds_relative_import_paths() {
    assert_eq!(
        relative_path("import Foo from './Foo.vue';"),
        Some("./Foo.vue")
    );
    assert_eq!(
        relative_path("import type { Foo } from \"../types\";"),
        Some("../types")
    );
}

#[test]
fn ignores_non_relative_import_paths() {
    assert_eq!(relative_path("import { ref } from 'vue';"), None);
}

#[test]
fn skips_from_inside_imported_names() {
    assert_eq!(
        relative_path("import { fromNow } from './time';"),
        Some("./time")
    );
}
