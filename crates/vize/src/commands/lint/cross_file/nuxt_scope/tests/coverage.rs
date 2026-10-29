use super::fixture::Fixture;

#[test]
fn unselected_and_new_untracked_importers_cannot_reuse_a_completeness_witness() {
    let mut fixture = Fixture::new();
    let mut analyzer = fixture.analyzer(&[]);
    assert!(fixture.prove(&mut analyzer).0);
    assert!(Fixture::ids(&mut analyzer).is_empty());

    fixture.source("src/pages/overview.vue",
        "<script setup>import Login from './login.vue'; import Signup from './signup.vue'</script><template><Login/><Signup/></template>");
    // Deliberately retain the old selected source set. The physical inventory
    // must find the newly added, untracked route without reading its content.
    let (proven, counts) = fixture.prove(&mut analyzer);
    assert!(!proven);
    assert_eq!(counts.read_dir, 3);
    assert_eq!(counts.manifest_reads, 5);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    let mut explicitly_omitted = fixture.analyzer(&["src/pages/overview.vue"]);
    assert!(!fixture.prove(&mut explicitly_omitted).0);
    assert_eq!(Fixture::ids(&mut explicitly_omitted).len(), 2);
}

#[test]
fn rc_project_ancestor_home_and_xdg_inputs_are_observed_without_content_reads() {
    for role in ["project", "ancestor", "home", "xdg"] {
        let fixture = Fixture::new();
        let target = match role {
            "project" => fixture.root.join(".nuxtrc"),
            "ancestor" => std::fs::canonicalize(fixture.temp.path())
                .unwrap()
                .join(".nuxtrc"),
            "home" => fixture.home.join(".nuxtrc"),
            _ => fixture.home.join(".config/.nuxtrc"),
        };
        std::fs::write(&target, b"app.pageTransition={\"mode\":\"in-out\"}\n").unwrap();
        let mut analyzer = fixture.analyzer(&[]);
        let (proven, counts) = fixture.prove(&mut analyzer);
        assert!(!proven, "{role}");
        assert_eq!(counts.manifest_reads, 5);
        assert_eq!(counts.manifest_bytes, fixture.manifest_bytes());
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    }
}

#[test]
fn failed_requalification_clears_a_previously_pending_host_claim() {
    let fixture = Fixture::new();
    let mut analyzer = fixture.analyzer(&[]);
    assert!(fixture.prove(&mut analyzer).0);
    let rc = fixture.home.join(".nuxtrc");
    std::fs::write(&rc, b"app.layoutTransition=true\n").unwrap();
    assert!(!fixture.prove(&mut analyzer).0);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    std::fs::remove_file(rc).unwrap();
    let (proven, counts) = fixture.prove(&mut analyzer);
    assert!(proven);
    assert_eq!(counts.manifest_reads, 5);
    assert!(Fixture::ids(&mut analyzer).is_empty());
}

#[test]
fn implicit_inputs_shadowing_nested_or_non_vue_entries_remain_unknown() {
    for name in [
        "src/router.options.ts",
        "src/app.config.ts",
        "plugins/runtime.ts",
        "modules/runtime.ts",
        "layers/custom/nuxt.config.ts",
        ".env",
        "src/components/NuxtPage.vue",
        "src/pages/README.md",
        "src/pages/nested/child.vue",
        "src/pages/Upper.vue",
    ] {
        let fixture = Fixture::new();
        fixture.write(name, b"opaque input");
        let mut analyzer = fixture.analyzer(&[]);
        let (proven, counts) = fixture.prove(&mut analyzer);
        assert!(!proven, "{name}");
        assert!(counts.read_dir <= 4);
        assert_eq!(counts.manifest_bytes, fixture.manifest_bytes());
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2, "{name}");
    }
}

#[test]
fn env_override_missing_home_relative_xdg_and_partial_dtos_never_become_admitted() {
    let fixture = Fixture::new();
    for role in ["override", "missing-home", "relative-xdg"] {
        let mut environment = fixture.environment();
        match role {
            "override" => environment.opaque = true,
            "missing-home" => environment.home = None,
            _ => environment.xdg = Some("relative-config".into()),
        }
        let mut analyzer = fixture.analyzer(&[]);
        assert!(
            !fixture
                .prove_with_environment(&mut analyzer, &environment)
                .0,
            "{role}"
        );
        assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    }
    let mut erased = fixture.analyzer_erasing(&[], true);
    let (proven, counts) = fixture.prove(&mut erased);
    assert!(!proven);
    assert_eq!(counts.manifest_reads, 0);
    erased.admit_nuxt_render_scope_once(true);
    assert_eq!(Fixture::ids(&mut erased).len(), 2);
}

#[cfg(unix)]
#[test]
fn source_and_rc_links_are_unknown_even_when_package_links_are_supported() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    let rc = fixture.home.join(".nuxtrc");
    symlink("missing-rc", &rc).unwrap();
    let mut analyzer = fixture.analyzer(&[]);
    assert!(!fixture.prove(&mut analyzer).0);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
    std::fs::remove_file(rc).unwrap();
    let page = fixture.root.join("src/pages/login.vue");
    let physical = fixture.root.join("src/pages/actual.vue");
    std::fs::rename(&page, &physical).unwrap();
    symlink(&physical, &page).unwrap();
    let mut analyzer = fixture.analyzer(&[]);
    assert!(!fixture.prove(&mut analyzer).0);
    assert_eq!(Fixture::ids(&mut analyzer).len(), 2);
}
