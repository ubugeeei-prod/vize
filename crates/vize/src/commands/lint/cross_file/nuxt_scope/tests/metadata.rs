use super::fixture::Fixture;
use crate::commands::project_nuxt::host_authority::io::{MAX_MANIFEST_BYTES, ProbeIo};

#[test]
fn original_source_facts_and_counted_complete_inventory_are_required() {
    let fixture = Fixture::new();
    let mut analyzer = fixture.analyzer(&[]);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    let (proven, counts) = fixture.prove(&mut analyzer);
    assert!(
        proven,
        "{counts:?}; facts={:?}",
        analyzer
            .registry()
            .vue_components()
            .map(|entry| (
                &entry.path,
                entry.analysis.nuxt_static_render_script,
                entry.analysis.nuxt_render_template,
                &entry.analysis.component_usages
            ))
            .collect::<Vec<_>>()
    );
    assert_eq!(counts.read_dir, 4);
    assert_eq!(counts.entries, 11);
    assert_eq!(counts.file_types, 11);
    assert_eq!(counts.manifest_opens, 5);
    assert_eq!(counts.manifest_reads, 5);
    assert_eq!(counts.manifest_parses, 5);
    assert_eq!(counts.manifest_bytes, fixture.manifest_bytes());
    assert_eq!(counts.package_lookups, 15);
    assert_eq!(counts.canonicalize, 15);
    assert!(Fixture::ids(&mut analyzer).is_empty());
    // The host witness cannot silently survive another analysis invocation.
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
}

#[test]
fn nested_kit_winner_is_not_replaced_by_a_convenient_hoisted_copy() {
    for nested_supported in [false, true] {
        let fixture = Fixture::new();
        let nested = fixture
            .root
            .join("node_modules/nuxt/node_modules/@nuxt/kit");
        let top = fixture.root.join("node_modules/@nuxt/kit");
        fixture.package(
            &nested,
            "@nuxt/kit",
            if nested_supported { "4.6.3" } else { "99.0.0" },
        );
        fixture.package(
            &top,
            "@nuxt/kit",
            if nested_supported { "99.0.0" } else { "4.6.3" },
        );
        let mut analyzer = fixture.analyzer(&[]);
        let (proven, _) = fixture.prove(&mut analyzer);
        assert_eq!(proven, nested_supported);
        assert_eq!(Fixture::ids(&mut analyzer).is_empty(), nested_supported);
    }
}

#[test]
fn schema_and_rc9_are_resolved_from_the_actual_kit_context() {
    for package in ["@nuxt/schema", "rc9"] {
        let fixture = Fixture::new();
        let nested = fixture
            .root
            .join("node_modules/@nuxt/kit/node_modules")
            .join(package);
        fixture.package(&nested, package, "99.0.0");
        let mut analyzer = fixture.analyzer(&[]);
        assert!(!fixture.prove(&mut analyzer).0, "{package}");
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2, "{package}");
    }
}

#[test]
fn missing_first_winner_does_not_fall_through_to_an_ancestor_version() {
    let fixture = Fixture::new();
    std::fs::remove_file(fixture.root.join("node_modules/nuxt/package.json")).unwrap();
    fixture.package(
        &fixture.root.parent().unwrap().join("node_modules/nuxt"),
        "nuxt",
        "4.6.3",
    );
    let mut analyzer = fixture.analyzer(&[]);
    let (proven, counts) = fixture.prove(&mut analyzer);
    assert!(!proven);
    assert_eq!(counts.manifest_reads, 1);
    assert_eq!(counts.read_dir, 0);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
}

#[test]
fn unknown_exports_bundled_loader_versions_and_project_aliases_are_refused() {
    for role in ["exports", "subpath", "loader", "project-alias"] {
        let fixture = Fixture::new();
        if role == "project-alias" {
            fixture.json(
                "package.json",
                serde_json::json!({
                    "name": "fixture", "imports": {"#nuxt": "./custom.mjs"},
                }),
            );
        } else {
            let path = fixture.root.join("node_modules/@nuxt/kit/package.json");
            let mut value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            if role == "exports" {
                value["exports"]["."] = serde_json::json!({
                    "custom": "./custom.mjs", "default": "./dist/index.mjs",
                });
            } else if role == "subpath" {
                value["exports"]["./internal"] = "./alternate.mjs".into();
            } else {
                value["inlinedDependencies"]["c12"] = "99.0.0".into();
            }
            std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let mut analyzer = fixture.analyzer(&[]);
        assert!(!fixture.prove(&mut analyzer).0, "{role}");
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    }
}

#[test]
fn manifest_bytes_and_directory_entries_have_actual_read_bounds() {
    let fixture = Fixture::new();
    fixture.write("package.json", &vec![b' '; MAX_MANIFEST_BYTES as usize + 1]);
    let mut analyzer = fixture.analyzer(&[]);
    let (proven, counts) = fixture.prove(&mut analyzer);
    assert!(!proven);
    assert_eq!(counts.manifest_opens, 0);
    assert_eq!(counts.manifest_reads, 0);
    assert_eq!(counts.manifest_bytes, 0);

    let directory = fixture.root.join("large");
    std::fs::create_dir(&directory).unwrap();
    for index in 0..4097 {
        std::fs::write(directory.join(vize_l0::cstr!("item-{index}").as_str()), b"").unwrap();
    }
    let mut io = ProbeIo::default();
    assert!(io.directory(&directory).is_none());
    assert_eq!(io.counts.read_dir, 1);
    assert_eq!(io.counts.entries, 4097);
    assert_eq!(io.counts.file_types, 4096);
    assert!(io.directory(&directory).is_none());
    assert_eq!(io.counts.read_dir, 1);
}

#[cfg(unix)]
#[test]
fn verified_package_links_follow_physical_dependency_contexts() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let original = fixture.root.join("node_modules/nuxt");
    let physical = std::fs::canonicalize(fixture.temp.path())
        .unwrap()
        .join("store/nuxt");
    std::fs::create_dir_all(physical.parent().unwrap()).unwrap();
    std::fs::rename(&original, &physical).unwrap();
    symlink(&physical, &original).unwrap();
    std::fs::create_dir_all(physical.join("node_modules/@nuxt")).unwrap();
    symlink(
        fixture.root.join("node_modules/@nuxt/kit"),
        physical.join("node_modules/@nuxt/kit"),
    )
    .unwrap();
    symlink(
        fixture.root.join("node_modules/@nuxt/schema"),
        physical.join("node_modules/@nuxt/schema"),
    )
    .unwrap();
    let mut analyzer = fixture.analyzer(&[]);
    let (proven, counts) = fixture.prove(&mut analyzer);
    assert!(proven, "{counts:?}");
    assert_eq!(counts.manifest_reads, 5);
    assert!(Fixture::ids(&mut analyzer).is_empty());
}

#[test]
fn identical_root_and_src_directories_are_read_once() {
    let mut fixture = Fixture::new();
    for name in ["app.vue", "pages", "layouts"] {
        std::fs::rename(fixture.root.join("src").join(name), fixture.root.join(name)).unwrap();
    }
    std::fs::remove_dir(fixture.root.join("src")).unwrap();
    fixture.sources = fixture
        .sources
        .iter()
        .map(|name| name.strip_prefix("src/").unwrap().into())
        .collect();
    fixture.write(
        "nuxt.config.ts",
        b"export default defineNuxtConfig({ssr:false,srcDir:'.'});\n",
    );
    let mut analyzer = fixture.analyzer(&[]);
    let (proven, counts) = fixture.prove(&mut analyzer);
    assert!(proven, "{counts:?}");
    assert_eq!(counts.read_dir, 3);
    assert!(Fixture::ids(&mut analyzer).is_empty());
}

#[test]
fn a_config_borrowed_from_an_ancestor_does_not_authenticate_a_child_invocation() {
    let fixture = Fixture::new();
    let nested = fixture.root.join("src");
    let dirs = crate::commands::project_nuxt::render_dirs(&nested).unwrap();
    assert!(!dirs.static_render_config);
}

#[test]
fn exported_dist_importers_take_precedence_over_package_root_copies() {
    for (parent, package, version) in [
        ("nuxt", "@nuxt/kit", "99.0.0"),
        ("@nuxt/kit", "@nuxt/schema", "99.0.0"),
        ("@nuxt/kit", "rc9", "99.0.0"),
    ] {
        let fixture = Fixture::new();
        let nested = fixture
            .root
            .join("node_modules")
            .join(parent)
            .join("dist/node_modules")
            .join(package);
        fixture.package(&nested, package, version);
        let mut analyzer = fixture.analyzer(&[]);
        assert!(!fixture.prove(&mut analyzer).0, "{parent} -> {package}");
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    }
}

#[cfg(unix)]
#[test]
fn linked_export_directories_cannot_change_the_importer_context() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let original = fixture.root.join("node_modules/nuxt/dist");
    let outside = std::fs::canonicalize(fixture.temp.path())
        .unwrap()
        .join("external-dist");
    std::fs::rename(&original, &outside).unwrap();
    symlink(&outside, &original).unwrap();
    let mut analyzer = fixture.analyzer(&[]);
    assert!(!fixture.prove(&mut analyzer).0);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
}

#[test]
fn an_export_directory_package_scope_cannot_add_unobserved_self_references() {
    for package in ["nuxt", "@nuxt/kit"] {
        let fixture = Fixture::new();
        let path = fixture
            .root
            .join("node_modules")
            .join(package)
            .join("dist/package.json");
        std::fs::write(
            path,
            br#"{"name":"@nuxt/kit","type":"module","exports":{".":"./index.mjs"}}"#,
        )
        .unwrap();
        let mut analyzer = fixture.analyzer(&[]);
        assert!(!fixture.prove(&mut analyzer).0, "{package}");
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    }
}

#[test]
fn a_shared_probe_cannot_open_more_than_six_distinct_manifests() {
    let fixture = Fixture::new();
    let mut io = ProbeIo::default();
    for index in 0..7 {
        let root = fixture
            .root
            .join(vize_l0::cstr!("manifest-{index}").as_str());
        fixture.package(&root, "rc9", "3.1.0");
        assert_eq!(io.manifest(&root).is_some(), index < 6);
    }
    assert_eq!(io.counts.manifest_opens, 6);
    assert_eq!(io.counts.manifest_reads, 6);
    assert_eq!(io.counts.manifest_parses, 6);
}

#[test]
fn unreviewed_internal_import_aliases_refuse_even_a_supported_package_version() {
    let fixture = Fixture::new();
    let path = fixture.root.join("node_modules/nuxt/package.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    value["imports"]["#app"] = "./different-app.mjs".into();
    std::fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    let mut analyzer = fixture.analyzer(&[]);
    assert!(!fixture.prove(&mut analyzer).0);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
}
