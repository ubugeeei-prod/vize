use super::*;

fn write(root: &Path, relative: &str, content: &str) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn cached_package_registration_reports_nested_routes_once() {
    let root = tempfile::tempdir().unwrap();
    let entry = write(
        root.path(),
        "node_modules/widgets/index.d.ts",
        "import { button } from 'widget-dep';\nexport declare const widget = button;\n",
    );
    write(
        root.path(),
        "node_modules/widgets/package.json",
        r#"{"name":"widgets","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/widget-dep/package.json",
        r#"{"name":"widget-dep","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/widget-dep/index.d.ts",
        "export { default as button } from './Button.vue';\n",
    );
    write(
        root.path(),
        "node_modules/widget-dep/Button.vue",
        "<template><button /></template>\n",
    );

    let mut canonical_paths = CanonicalPathCache::default();
    let entry = canonical_paths.canonicalize(&entry);
    let mut packages = PackageRouteResolver::default();
    let mut cache = registration::VirtualRegistrationCache::default();
    let mut first = registration::VirtualRegistrationDiscovery::default();
    let mut second = registration::VirtualRegistrationDiscovery::default();

    assert!(
        registration::non_relative_import_needs_virtual_registration(
            &entry,
            &mut canonical_paths,
            ImportFileOptions::from(false),
            None,
            Some(&mut packages),
            &mut cache,
            &mut first,
        )
    );
    assert!(
        registration::non_relative_import_needs_virtual_registration(
            &entry,
            &mut canonical_paths,
            ImportFileOptions::from(false),
            None,
            Some(&mut packages),
            &mut cache,
            &mut second,
        )
    );

    assert_eq!(first.package_routes.len(), 1);
    assert_eq!(
        first.package_routes[0].specifier.as_str(),
        "widget-dep",
        "the nested package route is reported on the first miss"
    );
    assert!(
        first.package_routes[0].route.is_some(),
        "the nested route reaches a Vue source and needs a shadow"
    );
    assert!(
        second.package_routes.is_empty(),
        "cache hits must not replay clone-heavy nested route bindings"
    );
    assert_eq!(
        first.package_sources, second.package_sources,
        "caller-specific package routes still receive source invalidation inputs"
    );
    assert_eq!(cache.registration_entries(), 1);
}
