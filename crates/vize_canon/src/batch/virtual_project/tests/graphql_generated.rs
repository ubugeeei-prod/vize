use std::fs;

use super::{VirtualProject, unique_case_dir};
use vize_carton::cstr;

#[test]
fn materialized_project_keeps_generated_graphql_schema_on_real_path() {
    let case_dir = unique_case_dir("graphql-generated-real-path");
    let _ = fs::remove_dir_all(&case_dir);
    let src_dir = case_dir.join("src");
    let schema_path = case_dir.join("types/codegen/schema.ts");
    fs::create_dir_all(&src_dir).unwrap();
    fs::create_dir_all(schema_path.parent().unwrap()).unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": {
      "~/*": ["*"]
    }
  }
}"#,
    )
    .unwrap();
    fs::write(
        &schema_path,
        r#"// Generated GraphQL schema types.
export enum AimQuestionDisplayKind {
  Text = 'TEXT',
}

export type AimQuestion = {
  kind: AimQuestionDisplayKind
}
"#,
    )
    .unwrap();
    let question = src_dir.join("question.ts");
    fs::write(
        &question,
        r#"import type { AimQuestion } from '~/types/codegen/schema'

export function expectQuestion(question: AimQuestion): AimQuestion {
  return question
}
"#,
    )
    .unwrap();
    let schema_specifier = schema_path
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/");
    let app = src_dir.join("App.vue");
    fs::write(
        &app,
        cstr!(
            r#"<script setup lang="ts">
import {{ expectQuestion }} from './question'
import {{ AimQuestionDisplayKind, type AimQuestion }} from '{schema_specifier}'

const question = {{
  kind: AimQuestionDisplayKind.Text,
}} satisfies AimQuestion

expectQuestion(question)
</script>

<template><div /></template>
"#
        )
        .as_str(),
    )
    .unwrap();

    let mut project = VirtualProject::new(&case_dir).unwrap();
    project.set_tsconfig_path(Some(case_dir.join("tsconfig.json")));
    project
        .register_paths(&[app.clone(), question.clone()])
        .unwrap();
    project.materialize().unwrap();

    let virtual_root = project.virtual_root().to_path_buf();
    assert!(
        virtual_root.join("src/question.ts").exists(),
        "the TS helper that participates in the type comparison is still materialized"
    );
    assert!(
        !virtual_root.join("types/codegen/schema.ts").exists(),
        "generated GraphQL schema types must not be duplicated into .vize/canon"
    );
    assert!(
        project
            .find_by_original(&app)
            .unwrap()
            .content
            .contains(&schema_specifier),
        "the SFC keeps using the real generated schema module"
    );
    assert_eq!(
        fs::read_to_string(virtual_root.join("src/question.ts")).unwrap(),
        fs::read_to_string(&question).unwrap(),
        "the helper keeps the alias so the generated tsconfig fallback resolves the real schema"
    );

    let tsconfig_path = virtual_root.join("tsconfig.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(tsconfig_path).unwrap()).unwrap();
    assert_eq!(
        value["compilerOptions"]["paths"]["~/*"],
        serde_json::json!(["./*", "../../../*"])
    );

    let _ = fs::remove_dir_all(&case_dir);
}
