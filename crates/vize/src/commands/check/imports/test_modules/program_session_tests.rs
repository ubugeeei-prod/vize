use super::*;

fn write(root: &Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    path
}

fn package(root: &Path, name: &str, index: &str) {
    write(
        root,
        &cstr!("node_modules/{name}/package.json"),
        &cstr!(r#"{{"name":"{name}","exports":{{".":{{"types":"./index.d.ts"}}}}}}"#),
    );
    write(root, &cstr!("node_modules/{name}/index.d.ts"), index);
}

/// Nuxt's referenced programs resolve `#imports` per program. A module
/// runtime that imports it reaches Vue from the app but not from the server.
#[test]
fn referenced_programs_rescan_packages_only_where_their_aliases_disagree() {
    let case = tempfile::tempdir().unwrap();
    let root = case.path().canonicalize().unwrap();
    let root = root.as_path();
    let main = write(
        root,
        "main.ts",
        "import { module } from 'nuxt-module';\nexport { module };\n",
    );
    package(
        root,
        "nuxt-module",
        "export { runtime as module } from 'module-runtime';\n",
    );
    package(root, "module-runtime", "export * from '#imports';\n");
    write(root, "app-imports.ts", "export * from './Widget.vue';\n");
    write(root, "Widget.vue", "<template><div /></template>\n");
    write(root, "server-imports.ts", "export const runtime = 1;\n");

    let programs = [
        ("app", r##"{"#imports":["./app-imports.ts"]}"##),
        ("server", r##"{"#imports":["./server-imports.ts"]}"##),
        (
            "shared",
            r##"{"#imports":["./app-imports.ts"],"~/*":["./*"]}"##,
        ),
    ];
    let mut packages = PackageRouteResolver::default();
    let mut session = LocalImportSession::default();
    let mut scans = Vec::new();
    let mut shadowed = Vec::new();
    for (name, paths) in programs {
        let tsconfig = write(
            root,
            &cstr!("tsconfig.{name}.json"),
            &cstr!(r#"{{"compilerOptions":{{"paths":{paths}}}}}"#),
        );
        let aliases = PathAliasResolver::from_tsconfig(Some(&tsconfig));
        session.begin_program(&mut packages);
        let shared = collect_transitive_local_imports_with_session(
            std::slice::from_ref(&main),
            root,
            &mut CanonicalPathCache::default(),
            false,
            Some(&aliases),
            &mut packages,
            &mut session,
        );
        let mut fresh_packages = PackageRouteResolver::default();
        let mut fresh_session = LocalImportSession::new(&mut fresh_packages);
        let fresh = collect_transitive_local_imports_with_session(
            std::slice::from_ref(&main),
            root,
            &mut CanonicalPathCache::default(),
            false,
            Some(&aliases),
            &mut fresh_packages,
            &mut fresh_session,
        );
        assert_eq!(shared.registrations, fresh.registrations, "{name}");
        assert_eq!(shared.authored, fresh.authored, "{name}");
        assert_eq!(shared.package_routes, fresh.package_routes, "{name}");
        scans.push(session.registration_cache.finished_scans());
        shadowed.push(
            shared
                .package_routes
                .iter()
                .filter(|binding| binding.route.is_some())
                .map(|binding| binding.specifier.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    assert_eq!(
        shadowed,
        [
            "nuxt-module,module-runtime",
            "",
            "nuxt-module,module-runtime"
        ]
    );
    // The server answers `#imports` differently; `shared` agrees with the app.
    assert_eq!(scans, [1, 2, 2]);
}
