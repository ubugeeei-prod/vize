//! Original #7991 private TS imports retain one authored package boundary.

use std::path::Path;

use crate::corsa_bridge::vue_dependencies_alias::AliasContext;

const CORPUS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/package-private-imports/"
);

fn fixture(root: &Path) {
    for (source, target) in [
        ("package.json.txt", "package.json"),
        ("tsconfig.json.txt", "tsconfig.json"),
        ("util.ts.txt", "src/lib/util.ts"),
        ("App.vue.txt", "src/App.vue"),
        ("main.ts.txt", "src/main.ts"),
    ] {
        let target = root.join(target);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(Path::new(CORPUS).join(source), target).unwrap();
    }
}

#[test]
fn original_private_ts_imports_materialize_manifest_and_target_for_both_hosts() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    fixture(&root);
    for file in ["src/App.vue", "src/main.ts"] {
        let host = root.join(file);
        let source = std::fs::read_to_string(&host).unwrap();
        let context = AliasContext::for_host(&host, &source, &Default::default());
        let mirror = context.mirror.as_ref().unwrap();
        assert!(context.aliases.is_empty());
        assert_eq!(context.package_routes.len(), 1);
        let route =
            &context.package_routes[&(root.join("src"), vize_carton::String::from("#lib/util.ts"))];
        assert_eq!(route.manifest_path, root.join("package.json"));
        assert_eq!(route.source_paths, [root.join("src/lib/util.ts")]);
        let mut expected = vec![host, root.join("src/lib/util.ts")];
        expected.sort();
        assert_eq!(mirror.registered_original_paths_sorted(), expected);
        mirror.materialize().unwrap();
        assert_eq!(
            std::fs::read(mirror.virtual_root().join("package.json")).unwrap(),
            std::fs::read(root.join("package.json")).unwrap()
        );
        assert_eq!(
            std::fs::read(mirror.virtual_root().join("src/lib/util.ts")).unwrap(),
            std::fs::read(root.join("src/lib/util.ts")).unwrap()
        );
        assert!(!root.join("vize.config.json").exists());
    }
}

#[test]
fn ordinary_native_ts_packages_and_missing_private_targets_do_not_gain_shadows() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    fixture(&root);
    let package = root.join("node_modules/ordinary");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"ordinary","exports":"./index.ts","type":"module"}"#,
    )
    .unwrap();
    std::fs::copy(root.join("src/lib/util.ts"), package.join("index.ts")).unwrap();
    let host = root.join("src/App.vue");
    let original = std::fs::read_to_string(&host).unwrap();
    for specifier in ["ordinary", "#lib/missing.ts"] {
        let source = original.replace("#lib/util.ts", specifier);
        let context = AliasContext::for_host(&host, &source, &Default::default());
        assert!(context.aliases.is_empty());
        assert!(context.package_routes.is_empty());
        assert_eq!(
            context
                .mirror
                .as_ref()
                .unwrap()
                .registered_original_paths_sorted(),
            [host.clone()]
        );
        assert!(!context.route_inputs.is_empty());
    }
}
