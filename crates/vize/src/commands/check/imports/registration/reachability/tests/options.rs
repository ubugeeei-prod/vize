use super::{CanonicalPathCache, ImportFileOptions, PathAliasResolver, resolve_local, write};

fn options_with(include_js: bool, include_jsx: bool) -> ImportFileOptions {
    ImportFileOptions {
        include_js,
        include_jsx,
    }
}

#[test]
fn enabled_javascript_options_preserve_resolution_and_recording_boundaries() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap();
    let importer = write(&root, "entry.ts", "export {};\n");
    let runtime = write(&root, "runtime.js", "export const value = 1;\n");
    let jsx = write(&root, "view.jsx", "export const value = <div />;\n");
    let declaration = write(
        &root,
        "neighbor.d.ts",
        "export declare const value: number;\n",
    );
    write(
        &root,
        "neighbor.js",
        "export const value = 'wrong declaration';\n",
    );
    let tsx = write(&root, "component.tsx", "export const value = <div />;\n");
    let js = write(&root, "component.js", "export const value = 1;\n");
    let config = write(
        &root,
        "tsconfig.json",
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["missing/*","*"]}}}"#,
    );
    let aliases = PathAliasResolver::from_tsconfig(Some(&config));
    for include_js in [false, true] {
        for include_jsx in [false, true] {
            let options = options_with(include_js, include_jsx);
            let cases = [
                ("./runtime.js", include_js.then_some(runtime.as_path())),
                ("./runtime", include_js.then_some(runtime.as_path())),
                (
                    "./view.jsx",
                    (include_js || include_jsx).then_some(jsx.as_path()),
                ),
                (
                    "./view",
                    (include_js || include_jsx).then_some(jsx.as_path()),
                ),
                ("./neighbor.js", Some(declaration.as_path())),
                ("./neighbor", Some(declaration.as_path())),
                (
                    "./component.js",
                    if include_jsx {
                        Some(tsx.as_path())
                    } else {
                        include_js.then_some(js.as_path())
                    },
                ),
            ];
            for (specifier, expected) in cases {
                let tail = specifier.strip_prefix("./").unwrap();
                let absolute = root.join(tail);
                let aliased = vize_l0::cstr!("@/{tail}");
                for (spelling, retained_alias) in [
                    (specifier, false),
                    (absolute.to_str().unwrap(), false),
                    (aliased.as_str(), true),
                ] {
                    let selected_aliases = retained_alias.then_some(&aliases);
                    let tracked = resolve_local::<true>(
                        &importer,
                        spelling,
                        &mut CanonicalPathCache::default(),
                        options,
                        selected_aliases,
                    );
                    let untracked = resolve_local::<false>(
                        &importer,
                        spelling,
                        &mut CanonicalPathCache::default(),
                        options,
                        selected_aliases,
                    );
                    assert_eq!(
                        tracked.0.as_deref(),
                        expected,
                        "{spelling}; include_js={include_js}; include_jsx={include_jsx}"
                    );
                    assert_eq!(untracked.0, tracked.0);
                    assert!(!tracked.1.is_empty());
                    if retained_alias {
                        assert_eq!(untracked.1, tracked.1);
                    } else {
                        assert!(untracked.1.is_empty());
                    }
                }
            }
        }
    }
}
