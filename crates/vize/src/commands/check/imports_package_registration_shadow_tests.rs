use super::*;

fn write(dir: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
#[cfg(unix)]
fn symlinked_workspace_build_output_package_gets_a_shadow_route() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let entry = write(
        &app,
        "src/entry.ts",
        "import { packed } from 'misskey-js';\nvoid packed;\n",
    );
    let package = root.path().join("packages/misskey-js");
    let source = write(
        root.path(),
        "packages/misskey-js/src/index.ts",
        "export const packed: string = 'ok';\n",
    );
    write(
        root.path(),
        "packages/misskey-js/package.json",
        r#"{
  "type": "module",
  "name": "misskey-js",
  "main": "./built/index.js",
  "types": "./built/index.d.ts",
  "exports": {
    ".": {
      "import": "./built/index.js",
      "types": "./built/index.d.ts",
      "default": "./built/index.js"
    }
  }
}
"#,
    );
    let link = app.join("node_modules/misskey-js");
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    symlink(&package, &link).unwrap();

    let mut resolver = PackageRouteResolver::default();
    let discovered = collect_transitive_local_imports_with_resolver(
        &[entry],
        &app,
        &mut CanonicalPathCache::default(),
        false,
        None,
        &mut resolver,
    );

    assert!(discovered.registrations.is_empty());
    assert!(
        discovered
            .authored
            .contains(&source.canonicalize().unwrap())
    );
    assert_eq!(discovered.package_routes.len(), 1);
    let route = discovered.package_routes[0]
        .route
        .as_ref()
        .expect("workspace build fallback must be materialized");
    assert!(route.requires_workspace_source_shadow());
    assert_eq!(
        route.unambiguous_source_path(),
        Some(&source.canonicalize().unwrap())
    );
}
