use super::super::imports_aliases::PathAliasResolver;
use super::*;
use vize_carton::path::canonicalize_non_verbatim;

fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn skips_plain_absolute_generated_graphql_imports() {
    let root =
        std::env::temp_dir().join(cstr!("vize-imports-abs-generated-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    let schema = write(
        &root,
        "types/codegen/schema.ts",
        "// Generated GraphQL schema types.\nexport enum AimQuestionDisplayKind { Text = 'TEXT' }\nexport type AimQuestion = { kind: AimQuestionDisplayKind }\n",
    );
    let schema_specifier = schema.with_extension("");
    let entry = write(
        &root,
        "src/entry.ts",
        cstr!(
            "import type {{ AimQuestion }} from '{}'\nexport type Props = {{ question: AimQuestion }}\n",
            schema_specifier.display()
        )
        .as_str(),
    );

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );

    assert!(
        discovered.is_empty(),
        "plain absolute generated type modules should resolve from their real path, not be copied into .vize/canon: {discovered:#?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn skips_plain_aliased_generated_graphql_imports() {
    let root =
        std::env::temp_dir().join(cstr!("vize-imports-alias-generated-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
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
    write(
        &root,
        "types/codegen/schema.ts",
        "// Generated GraphQL schema types.\nexport enum AimQuestionDisplayKind { Text = 'TEXT' }\n",
    );
    let entry = write(
        &root,
        "src/entry.ts",
        "import type { AimQuestionDisplayKind } from '~/types/codegen/schema'\nexport type Props = { kind: AimQuestionDisplayKind }\n",
    );
    let aliases = PathAliasResolver::from_tsconfig(Some(&root.join("tsconfig.json")));

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        Some(&aliases),
    );

    assert!(
        discovered.is_empty(),
        "plain aliased generated type modules should use the real tsconfig path fallback, not be copied into .vize/canon: {discovered:#?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn collects_absolute_project_imports_that_need_vue_rewrites() {
    let root = std::env::temp_dir().join(cstr!("vize-imports-abs-vue-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    let widget = write(
        &root,
        "src/Widget.vue",
        "<script setup lang=\"ts\">const label = 'ok'</script>\n<template><div /></template>\n",
    );
    let feature = write(
        &root,
        "src/feature.ts",
        "import { Widget } from './nested'\nexport { Widget }\n",
    );
    let nested = write(
        &root,
        "src/nested.ts",
        "import Widget from './Widget.vue'\nexport { Widget }\n",
    );
    let feature_specifier = feature.with_extension("");
    let entry = write(
        &root,
        "src/entry.ts",
        cstr!(
            "import {{ Widget }} from '{}'\nexport {{ Widget }}\n",
            feature_specifier.display()
        )
        .as_str(),
    );

    let discovered = collect_transitive_local_imports(
        &[entry],
        &root,
        &mut CanonicalPathCache::default(),
        false,
        None,
    );

    assert_eq!(
        discovered,
        vec![
            canonicalize_non_verbatim(&feature),
            canonicalize_non_verbatim(&nested),
            canonicalize_non_verbatim(&widget)
        ]
    );

    let _ = std::fs::remove_dir_all(&root);
}
