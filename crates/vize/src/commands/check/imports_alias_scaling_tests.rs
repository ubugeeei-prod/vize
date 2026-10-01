use super::*;

fn write(root: &Path, relative: &str, content: &str) -> std::io::Result<PathBuf> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, content)?;
    Ok(path)
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
        include_str!("../../../tests/fixtures/import-registration/Child.vue"),
    )
    .unwrap();
    let mut paths = CanonicalPathCache::default();
    let mut cache = registration::VirtualRegistrationCache::default();
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
    assert_eq!(cache.source_reads(), 2);
    std::fs::write(&leaf, "export { default } from './Child.vue';").unwrap();
    let mut next = registration::VirtualRegistrationCache::default();
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
    let child = write(
        root,
        "Child.vue",
        include_str!("../../../tests/fixtures/import-registration/Child.vue"),
    )
    .unwrap();
    let mut paths = CanonicalPathCache::default();
    let mut cache = registration::VirtualRegistrationCache::default();
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
    assert!(
        discovery
            .package_sources
            .contains(&paths.canonicalize(&leaf))
    );
    assert!(child.exists());
    assert_eq!(
        cache.source_reads(),
        3,
        "package walks still collect transitive invalidation inputs"
    );
}
