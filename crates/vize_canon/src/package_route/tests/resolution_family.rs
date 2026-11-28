//! Inactive export targets must not replace real legacy declaration inputs.

use super::super::{
    PackageResolutionContext, PackageResolutionMode, PackageRouteResolver, PackageSourceOptions,
};

#[test]
#[cfg(unix)]
fn legacy_resolution_keeps_existing_subpath_declarations_without_export_source_fallbacks() {
    let root = tempfile::tempdir().unwrap();
    let importer = root.path().join("app/src");
    let package = root.path().join("packages/sandbox");
    let link = root.path().join("app/node_modules/sandbox");
    for directory in [
        importer.clone(),
        link.parent().unwrap().to_path_buf(),
        package.join("src/web"),
        package.join("dist/web"),
    ] {
        std::fs::create_dir_all(directory).unwrap();
    }
    std::os::unix::fs::symlink(&package, &link).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"sandbox","exports":{"./web":{"types":"./dist/web.d.ts"}}}"#,
    )
    .unwrap();
    let declaration = package.join("web.d.ts");
    std::fs::write(&declaration, "export * from './dist/web/index.d.ts';\n").unwrap();
    std::fs::write(
        package.join("dist/web/index.d.ts"),
        "export declare function run(): string;\n",
    )
    .unwrap();
    let source = package.join("src/web/index.ts");
    std::fs::write(&source, "export { run } from '~/worker';\n").unwrap();
    let declaration = declaration.canonicalize().unwrap();
    let source = source.canonicalize().unwrap();
    let mut resolver = PackageRouteResolver::default();

    for family in ["node", "node10", "classic"] {
        let context = PackageResolutionContext::new(
            Some(family),
            PackageResolutionMode::Import,
            std::iter::empty::<&str>(),
        );
        let serial = resolver.lookup_with_context(
            &importer,
            "sandbox/web",
            PackageSourceOptions::default(),
            context.clone(),
        );
        let route = serial.route().unwrap();
        assert_eq!(route.source_paths, [declaration.clone()]);
        assert!(
            route
                .source_targets
                .iter()
                .all(|target| target.source_path != source)
        );
        // The concurrent miss must use the same family, not only reuse the
        // serial cached answer.
        let concurrent = PackageRouteResolver::default().lookup_with_context_concurrent(
            &importer,
            "sandbox/web",
            PackageSourceOptions::default(),
            context,
        );
        assert_eq!(serial, concurrent);
    }

    for family in ["bundler", "node16", "nodenext"] {
        let modern = resolver.lookup_with_context(
            &importer,
            "sandbox/web",
            PackageSourceOptions::default(),
            PackageResolutionContext::new(
                Some(family),
                PackageResolutionMode::Import,
                std::iter::empty::<&str>(),
            ),
        );
        assert_eq!(
            modern.route().unwrap().source_paths,
            [source.clone(), declaration.clone()]
        );
    }
}
