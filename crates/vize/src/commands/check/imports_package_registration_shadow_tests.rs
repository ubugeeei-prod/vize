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
    let mut canonical_paths = CanonicalPathCache::default();
    let mut session = LocalImportSession::new(&mut resolver);
    let discovered = collect_transitive_local_imports_with_session(
        &[entry],
        &app,
        &mut canonical_paths,
        false,
        None,
        &mut resolver,
        &mut session,
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

#[test]
#[cfg(unix)]
fn package_self_import_does_not_report_sibling_sources() {
    let root = tempfile::tempdir().unwrap();
    let package = root.path().join("packages/wave-ui");
    let sibling = write(
        root.path(),
        "packages/wave-ui/src/sibling.ts",
        "export const extra = 2;\n",
    );
    write(
        root.path(),
        "packages/wave-ui/src/index.ts",
        "export const value = 1;\nexport * from './sibling';\n",
    );
    write(
        root.path(),
        "packages/wave-ui/package.json",
        r#"{
  "type": "module",
  "name": "wave-ui",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": { ".": { "import": "./dist/index.js", "types": "./dist/index.d.ts" } }
}
"#,
    );
    let entry = write(
        root.path(),
        "packages/wave-ui/src/docs.ts",
        "import { value } from 'wave-ui';\nvoid value;\n",
    );

    let mut resolver = PackageRouteResolver::default();
    let mut canonical_paths = CanonicalPathCache::default();
    let mut session = LocalImportSession::new(&mut resolver);
    let discovered = collect_transitive_local_imports_with_session(
        &[entry],
        &package,
        &mut canonical_paths,
        false,
        None,
        &mut resolver,
        &mut session,
    );

    assert!(
        !discovered
            .authored
            .contains(&sibling.canonicalize().unwrap()),
        "a file inside the package must not re-check the package source tree"
    );
}
