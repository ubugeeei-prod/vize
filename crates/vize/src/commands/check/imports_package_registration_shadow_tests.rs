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

#[test]
#[cfg(unix)]
fn absent_vue_package_candidates_are_not_authored_members() {
    use std::os::unix::fs::symlink;

    const APP: &str = include_str!(
        "../../../../../tests/_fixtures/differential/compat/package-candidate-membership/ViteApp.vue.txt"
    );
    let originals = [
        (
            "packages/ui/package.json",
            include_str!(
                "../../../../../tests/_fixtures/differential/compat/package-candidate-membership/ui-package-typed.json.txt"
            ),
        ),
        (
            "packages/ui/src/BadgeCard.vue",
            include_str!(
                "../../../../../tests/_fixtures/differential/compat/package-candidate-membership/BadgeCard.vue.txt"
            ),
        ),
        (
            "packages/pricing/package.json",
            include_str!(
                "../../../../../tests/_fixtures/differential/compat/package-candidate-membership/pricing-package.json.txt"
            ),
        ),
        (
            "packages/pricing/src/index.js",
            include_str!(
                "../../../../../tests/_fixtures/differential/compat/package-candidate-membership/pricing-index.js.txt"
            ),
        ),
        (
            "packages/pricing/src/label.mjs",
            include_str!(
                "../../../../../tests/_fixtures/differential/compat/package-candidate-membership/pricing-label.mjs.txt"
            ),
        ),
    ];
    for missing in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("apps/vite");
        for (name, source) in originals {
            write(root.path(), name, source);
        }
        for name in ["ui", "pricing"] {
            let link = app.join("node_modules/@workspace").join(name);
            std::fs::create_dir_all(link.parent().unwrap()).unwrap();
            symlink(root.path().join("packages").join(name), link).unwrap();
        }
        let source = if missing {
            APP.replace("@workspace/ui/BadgeCard.vue", "@workspace/ui/Missing.vue")
        } else {
            APP.to_owned()
        };
        let entry = write(&app, "src/App.vue", &source);
        let mut resolver = PackageRouteResolver::default();
        let mut session = LocalImportSession::new(&mut resolver);
        let discovered = collect_transitive_local_imports_with_session(
            std::slice::from_ref(&entry),
            &app,
            &mut CanonicalPathCache::default(),
            ImportFileOptions {
                include_js: true,
                include_jsx: false,
            },
            None,
            &mut resolver,
            &mut session,
        );
        let canonical = root.path().canonicalize().unwrap();
        let mut expected = vec![
            canonical.join("packages/pricing/src/index.js"),
            canonical.join("packages/pricing/src/label.mjs"),
        ];
        if !missing {
            expected.push(canonical.join("packages/ui/src/BadgeCard.vue"));
        }
        let mut actual = discovered.authored;
        actual.sort();
        assert_eq!(actual, expected);
        assert_eq!(discovered.registrations, Vec::<PathBuf>::new());
        let specifier = if missing {
            "@workspace/ui/Missing.vue"
        } else {
            "@workspace/ui/BadgeCard.vue"
        };
        let route = discovered
            .package_routes
            .iter()
            .find(|binding| binding.specifier == specifier)
            .and_then(|binding| binding.route.as_ref())
            .unwrap();
        let candidate = route.package_root.join(if missing {
            "Missing.vue"
        } else {
            "BadgeCard.vue"
        });
        assert!(!candidate.exists());
        assert!(route.source_paths.contains(&candidate));
        assert!(
            route
                .source_targets
                .iter()
                .any(|target| target.source_path == candidate
                    && target.native_probe_path == candidate.with_extension("d.vue.ts"))
        );
        for (name, source) in originals {
            assert_eq!(
                std::fs::read_to_string(root.path().join(name)).unwrap(),
                source
            );
        }
    }
}
