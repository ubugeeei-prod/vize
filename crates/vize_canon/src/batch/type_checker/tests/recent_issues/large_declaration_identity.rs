//! Large first-party declaration modules must keep one module identity (#6000).

use super::super::{BatchTypeChecker, relative_path, resolve_test_tsgo_binary, unique_case_dir};
use crate::batch::TypeChecker;
use vize_carton::{String, cstr};

type ReportedDiagnostic = (String, Option<u32>, String);

#[test]
fn aliased_large_declaration_module_and_absolute_import_share_identity() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }

    let project_root = unique_case_dir("issue-6000-large-declaration-identity");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    std::fs::create_dir_all(project_root.join("types")).unwrap();
    super::super::link_workspace_node_modules(&project_root).unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "baseUrl": ".",
    "paths": {
      "~/*": ["*"]
    },
    "noEmit": true
  },
  "include": ["src/**/*", "types/**/*.d.ts"]
}"#,
    )
    .unwrap();
    // `unique symbol` makes the duplicate module identity observable in a compact
    // fixture; generated schemas can hit the same path through deeper branded or
    // enum-bearing graphs.
    let mut schema = std::string::String::from(
        "export declare const QuestionBrand: unique symbol\n\
         export enum QuestionKind { Text = 'TEXT', Choice = 'CHOICE' }\n\
         export type Question = { readonly [QuestionBrand]: true; kind: QuestionKind; title: string }\n\
         export type Node0 = { question: Question }\n",
    );
    for index in 1..384 {
        schema.push_str(&format!(
            "export type Node{index} = {{ previous: Node{}; question: Question }}\n",
            index - 1
        ));
    }
    schema.push_str("export type GeneratedSchema = Node383\n");
    std::fs::write(project_root.join("types/schema.d.ts"), schema).unwrap();
    std::fs::write(
        project_root.join("src/question.ts"),
        r#"import type { Question } from '~/types/schema';

export function expectQuestion(question: Question): Question {
  return question;
}
"#,
    )
    .unwrap();
    let schema_specifier = project_root
        .join("types/schema")
        .to_string_lossy()
        .replace('\\', "/");
    std::fs::write(
        project_root.join("src/App.vue"),
        cstr!(
            r#"<script setup lang="ts">
import {{ expectQuestion }} from './question';
import {{ QuestionBrand, QuestionKind, type Question }} from '{schema_specifier}';

const question: Question = {{
  [QuestionBrand]: true,
  kind: QuestionKind.Text,
  title: 'Hello',
}};

expectQuestion(question);
</script>

<template><div /></template>
"#
        )
        .as_str(),
    )
    .unwrap();

    let mut checker = BatchTypeChecker::new(&project_root).expect("batch checker construction");
    checker.scan_project().expect("project scan");
    let result = checker.check_project().expect("project check");
    let mut diagnostics: Vec<ReportedDiagnostic> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                relative_path(&project_root, &diagnostic.file),
                diagnostic.code,
                diagnostic.message,
            )
        })
        .collect();
    diagnostics.sort();
    let _ = std::fs::remove_dir_all(&project_root);

    assert!(
        diagnostics.is_empty(),
        "a single generated declaration module must not be duplicated into unrelated identities: {diagnostics:#?}"
    );
}
