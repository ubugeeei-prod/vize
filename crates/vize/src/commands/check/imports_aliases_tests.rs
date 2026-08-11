use super::super::{
    imports::{
        collect_transitive_local_imports, resolve_import_base, resolve_import_base_with_inputs,
    },
    path_cache::CanonicalPathCache,
};
use super::PathAliasResolver;
use std::path::{Path, PathBuf};

fn write(root: &Path, rel: &str, contents: &str) -> PathBuf {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn resolves_wildcard_alias_to_vue_source() {
    let root = tempfile::tempdir().unwrap();
    let keyboard = write(
        root.path(),
        "src/keyboards/EnglishKeyboard.vue",
        "<template />",
    );
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"~/*":["*"]}}}"#,
    )
    .unwrap();
    let resolver =
        PathAliasResolver::from_tsconfig(Some(root.path().join("tsconfig.json").as_path()));
    let resolved = resolver.resolve(
        "~/src/keyboards/EnglishKeyboard.vue",
        &mut CanonicalPathCache::default(),
        false,
        resolve_import_base,
    );
    assert_eq!(resolved, Some(keyboard.canonicalize().unwrap()));
}

#[test]
fn exact_alias_does_not_match_prefix() {
    let root = tempfile::tempdir().unwrap();
    let entry = write(root.path(), "src/exact.ts", "export const exact = 1;");
    let prefix = write(root.path(), "src/prefix.ts", "export const prefix = 1;");
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@app":["src/exact.ts"],"@app/*":["src/*"]}}}"#,
    )
    .unwrap();
    let resolver =
        PathAliasResolver::from_tsconfig(Some(root.path().join("tsconfig.json").as_path()));
    let mut paths = CanonicalPathCache::default();
    assert_eq!(
        resolver.resolve("@app", &mut paths, false, resolve_import_base),
        Some(entry.canonicalize().unwrap())
    );
    assert_eq!(
        resolver.resolve("@app/prefix", &mut paths, false, resolve_import_base),
        Some(prefix.canonicalize().unwrap())
    );
}

#[test]
fn missing_alias_target_retains_every_probed_candidate() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@missing":["src/Missing"]}}}"#,
    )
    .unwrap();
    let resolver =
        PathAliasResolver::from_tsconfig(Some(root.path().join("tsconfig.json").as_path()));

    let (resolved, inputs) = resolver.resolve_with_inputs(
        "@missing",
        &mut CanonicalPathCache::default(),
        false,
        resolve_import_base_with_inputs,
    );

    assert!(resolved.is_none());
    assert!(inputs.iter().any(|path| path.ends_with("src/Missing.ts")));
    assert!(inputs.iter().any(|path| path.ends_with("src/Missing.vue")));
}

#[test]
fn package_context_uses_flattened_options_and_authored_module_mode() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("base.json"),
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"NodeNext","customConditions":["base"]}}"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"extends":"./base.json","compilerOptions":{"customConditions":["browser"]}}"#,
    )
    .unwrap();
    let resolver = PathAliasResolver::from_tsconfig(Some(&root.path().join("tsconfig.json")));
    let (context, _) = resolver.package_resolution_context(
        &mut vize_canon::PackageRouteResolver::default(),
        &root.path().join("src/entry.mts"),
        vize_canon::PackageResolutionMode::Import,
    );
    assert_eq!(context.module_resolution.as_deref(), Some("nodenext"));
    assert_eq!(context.mode, vize_canon::PackageResolutionMode::Import);
    assert_eq!(context.active_conditions, ["browser"]);
}

#[test]
fn extends_array_merges_resolution_fields_left_to_right() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join("resolution.json"),
        r#"{"compilerOptions":{"module":"NodeNext","moduleResolution":"NodeNext"}}"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("conditions.json"),
        r#"{"compilerOptions":{"customConditions":["browser"]}}"#,
    )
    .unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"extends":["./resolution.json","./conditions.json"]}"#,
    )
    .unwrap();
    let resolver = PathAliasResolver::from_tsconfig(Some(&root.path().join("tsconfig.json")));
    let (context, _) = resolver.package_resolution_context(
        &mut vize_canon::PackageRouteResolver::default(),
        &root.path().join("src/entry.mts"),
        vize_canon::PackageResolutionMode::Import,
    );
    assert_eq!(context.module_resolution.as_deref(), Some("nodenext"));
    assert_eq!(context.active_conditions, ["browser"]);
}

#[test]
fn collector_registers_tsconfig_alias_vue_dependencies() {
    let root = tempfile::tempdir().unwrap();
    let entry = write(
        root.path(),
        "src/Entry.vue",
        r#"<script lang="ts">
import EnglishKeyboard from "~/src/keyboards/EnglishKeyboard.vue";
void EnglishKeyboard;
</script>
"#,
    );
    let keyboard = write(
        root.path(),
        "src/keyboards/EnglishKeyboard.vue",
        "<template />",
    );
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"~/*":["*"]}}}"#,
    )
    .unwrap();
    let resolver =
        PathAliasResolver::from_tsconfig(Some(root.path().join("tsconfig.json").as_path()));
    let discovered = collect_transitive_local_imports(
        &[entry],
        root.path(),
        &mut CanonicalPathCache::default(),
        false,
        Some(&resolver),
    );
    assert_eq!(
        discovered.registrations,
        vec![keyboard.canonicalize().unwrap()]
    );
}
