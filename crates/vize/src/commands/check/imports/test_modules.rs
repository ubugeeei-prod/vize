use super::*;

mod package_cache_tests;
mod package_registration_cache_tests;
mod package_registration_shadow_tests;
mod package_registration_tests;
mod tests;

mod alias_scaling_tests {
    use super::*;

    fn write(root: &Path, relative: &str, content: &str) -> std::io::Result<PathBuf> {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        path.canonicalize()
    }

    #[test]
    fn issue_7324_alias_chain_collection_reads_each_module_once() {
        for sfc_count in [100, 300, 600] {
            let case = tempfile::tempdir().unwrap();
            let root = case.path();
            let module_count = sfc_count * 2;
            let tsconfig = write(
                root,
                "tsconfig.json",
                r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["src/*"]}}}"#,
            )
            .unwrap();
            for index in 0..module_count {
                let source = if index == 0 {
                    cstr!("export const value0 = 0;")
                } else {
                    cstr!(
                        "import {{ value{} }} from '@/lib/mod{}'; export const value{index} = value{};",
                        index - 1,
                        index - 1,
                        index - 1
                    )
                };
                write(root, &cstr!("src/lib/mod{index}.ts"), &source).unwrap();
            }
            let mut roots = Vec::new();
            for index in 0..sfc_count {
                let imports = (0..8)
                    .map(|offset| {
                        let module = (index * 7 + offset * 13) % module_count;
                        cstr!("import {{ value{module} }} from '@/lib/mod{module}';")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                roots.push(
                write(
                    root,
                    &cstr!("src/Comp{index}.vue"),
                    &cstr!(
                        "<script setup lang=\"ts\">{imports}</script><template><div /></template>"
                    ),
                )
                .unwrap(),
            );
            }
            let aliases = PathAliasResolver::from_tsconfig(Some(&tsconfig));
            let mut packages = PackageRouteResolver::default();
            let mut session = LocalImportSession::new(&mut packages);
            let discovered = collect_transitive_local_imports_with_session(
                &roots,
                root,
                &mut CanonicalPathCache::default(),
                false,
                Some(&aliases),
                &mut packages,
                &mut session,
            );
            assert!(discovered.registrations.is_empty());
            assert!(discovered.package_routes.is_empty());
            assert_eq!(discovered.authored.len(), module_count);
            assert_eq!(
                session.registration_cache.source_reads(),
                module_count,
                "{sfc_count} SFCs must read {module_count} modules once, rather than revisit every suffix"
            );
        }
    }

    #[test]
    fn cached_negative_branches_do_not_hide_vue_reachability_or_next_collection_edits() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path();
        let leaf = write(root, "leaf.ts", "export const leaf = 1;").unwrap();
        let positive = write(
            root,
            "barrel.ts",
            "export * from './leaf'; export { default } from './Child.vue';",
        )
        .unwrap();
        write(
            root,
            "Child.vue",
            include_str!("../../../../tests/fixtures/import-registration/Child.vue"),
        )
        .unwrap();
        let mut paths = CanonicalPathCache::default();
        let mut cache = LocalImportSession::default();
        let mut discovery = registration::VirtualRegistrationDiscovery::default();
        assert!(
            !registration::non_relative_import_needs_virtual_registration(
                &leaf,
                &mut paths,
                false.into(),
                None,
                None,
                &mut cache,
                &mut discovery
            )
        );
        assert!(
            registration::non_relative_import_needs_virtual_registration(
                &positive,
                &mut paths,
                false.into(),
                None,
                None,
                &mut cache,
                &mut discovery
            )
        );
        assert_eq!(cache.registration_cache.source_reads(), 2);
        std::fs::write(&leaf, "export { default } from './Child.vue';").unwrap();
        let mut next = LocalImportSession::default();
        assert!(
            registration::non_relative_import_needs_virtual_registration(
                &leaf,
                &mut CanonicalPathCache::default(),
                false.into(),
                None,
                None,
                &mut next,
                &mut registration::VirtualRegistrationDiscovery::default()
            )
        );
    }

    #[test]
    fn package_walk_does_not_reuse_alias_negative_invalidation_shortcuts() {
        let case = tempfile::tempdir().unwrap();
        let root = case.path();
        let leaf = write(root, "leaf.ts", "export const leaf = 1;").unwrap();
        let positive = write(
            root,
            "barrel.ts",
            "export * from './leaf'; export { default } from './Child.vue';",
        )
        .unwrap();
        write(
            root,
            "Child.vue",
            include_str!("../../../../tests/fixtures/import-registration/Child.vue"),
        )
        .unwrap();
        let mut paths = CanonicalPathCache::default();
        let mut cache = LocalImportSession::default();
        assert!(
            !registration::non_relative_import_needs_virtual_registration(
                &leaf,
                &mut paths,
                false.into(),
                None,
                None,
                &mut cache,
                &mut registration::VirtualRegistrationDiscovery::default()
            )
        );
        let mut discovery = registration::VirtualRegistrationDiscovery::default();
        assert!(
            registration::non_relative_import_needs_virtual_registration(
                &positive,
                &mut paths,
                false.into(),
                None,
                Some(&mut PackageRouteResolver::default()),
                &mut cache,
                &mut discovery
            )
        );
        discovery.package_sources.sort();
        assert_eq!(
            discovery.package_sources,
            [paths.canonicalize(&positive), paths.canonicalize(&leaf)]
        );
        assert_eq!(
            cache.registration_cache.source_reads(),
            3,
            "package walks still collect transitive invalidation inputs"
        );
    }
}

mod generated_tests {
    use super::super::super::imports_aliases::PathAliasResolver;
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
            discovered.registrations.is_empty(),
            "plain absolute generated type modules should resolve from their real path, not be copied into .vize/canon: {discovered:#?}"
        );
        assert_eq!(
            discovered.authored,
            vec![canonicalize_non_verbatim(&schema)]
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
        let schema = write(
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
            discovered.registrations.is_empty(),
            "plain aliased generated type modules should use the real tsconfig path fallback, not be copied into .vize/canon: {discovered:#?}"
        );
        assert_eq!(
            discovered.authored,
            vec![canonicalize_non_verbatim(&schema)]
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
            discovered.registrations,
            vec![
                canonicalize_non_verbatim(&feature),
                canonicalize_non_verbatim(&nested),
                canonicalize_non_verbatim(&widget)
            ]
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}

mod js_tests {
    use super::*;
    use vize_carton::path::canonicalize_non_verbatim;

    fn write(root: &Path, relative: &str, contents: &str) -> PathBuf {
        let path = root.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn allow_js_collects_the_imported_javascript_family() {
        let root =
            std::env::temp_dir().join(cstr!("vize-imports-js-family-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let entry = write(
            &root,
            "src/entry.ts",
            "import '../support/plain'\nimport '../support/esm.mjs'\nimport '../support/common.cjs'\nimport '../support/view.jsx'\n",
        );
        let expected = [
            write(&root, "support/plain.js", "const plain = true\n"),
            write(&root, "support/esm.mjs", "const esm = true\n"),
            write(&root, "support/common.cjs", "const common = true\n"),
            write(&root, "support/view.jsx", "const view = true\n"),
        ]
        .map(|path| canonicalize_non_verbatim(&path));

        let disabled = collect_transitive_local_imports(
            std::slice::from_ref(&entry),
            &root,
            &mut CanonicalPathCache::default(),
            ImportFileOptions::default(),
            None,
        );
        let enabled = collect_transitive_local_imports(
            &[entry],
            &root,
            &mut CanonicalPathCache::default(),
            ImportFileOptions {
                include_js: true,
                include_jsx: false,
            },
            None,
        );

        assert!(disabled.registrations.is_empty());
        assert_eq!(enabled.registrations, expected);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn allow_js_keeps_typescript_substitution_ahead_of_javascript() {
        let root =
            std::env::temp_dir().join(cstr!("vize-imports-js-precedence-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let entry = write(&root, "src/entry.ts", "import '../support/module.js'\n");
        write(&root, "support/module.js", "export const source = 'js'\n");
        let typescript = write(&root, "support/module.ts", "export const source = 'ts'\n");

        let discovered = collect_transitive_local_imports(
            &[entry],
            &root,
            &mut CanonicalPathCache::default(),
            ImportFileOptions {
                include_js: true,
                include_jsx: false,
            },
            None,
        );

        assert_eq!(
            discovered.registrations,
            vec![canonicalize_non_verbatim(&typescript)]
        );
        let _ = std::fs::remove_dir_all(root);
    }
}

mod magic_comments_tests {
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
    fn collects_dynamic_imports_with_magic_comments() {
        let root = std::env::temp_dir().join(cstr!("vize-imports-magic-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();

        let entry = write(
            &root,
            "src/entry.ts",
            r#"const panel = import(
  /* webpackChunkName: "settings-panel" */
  "./Panel.vue"
)
void panel
"#,
        );
        let panel = write(
            &root,
            "src/Panel.vue",
            "<script setup lang=\"ts\">\nconst ready = true\n</script>\n<template>{{ ready }}</template>\n",
        );

        let discovered = collect_transitive_local_imports(
            &[entry],
            &root,
            &mut CanonicalPathCache::default(),
            false,
            None,
        );

        assert_eq!(
            discovered.registrations,
            vec![canonicalize_non_verbatim(&panel)]
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
