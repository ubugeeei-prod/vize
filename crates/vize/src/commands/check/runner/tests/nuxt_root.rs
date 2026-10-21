use super::super::{
    CanonicalPathCache, resolve_nuxt_project_root, resolve_program_nuxt_project_root,
};
use super::unique_case_dir;

#[test]
fn resolves_parent_nuxt_mjs_root_for_explicit_tsconfig() {
    let project_root = unique_case_dir("nuxt-mjs-root");
    let _ = std::fs::remove_dir_all(&project_root);
    let config_dir = project_root.join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    let tsconfig = config_dir.join("tsconfig.json");
    std::fs::write(project_root.join("nuxt.config.mjs"), "export default {}").unwrap();
    std::fs::write(&tsconfig, "{}").unwrap();

    let resolved = resolve_nuxt_project_root(Some(&tsconfig), &project_root, &config_dir);

    assert_eq!(resolved, project_root);

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn mixed_external_inputs_keep_nuxt_support_for_the_cwd_program() {
    let case_root = unique_case_dir("mixed-nuxt-root");
    let _ = std::fs::remove_dir_all(&case_root);
    let cwd = case_root.join("app");
    let local = cwd.join("src/App.vue");
    let external = case_root.join("external/Repro.vue");
    std::fs::create_dir_all(local.parent().unwrap()).unwrap();
    std::fs::create_dir_all(external.parent().unwrap()).unwrap();
    std::fs::write(cwd.join("nuxt.config.ts"), "export default {}\n").unwrap();
    std::fs::write(&local, "<template />\n").unwrap();
    std::fs::write(&external, "<template />\n").unwrap();

    let resolved = resolve_program_nuxt_project_root(
        &cwd,
        &case_root,
        None,
        &[local, external],
        &mut CanonicalPathCache::default(),
    );

    assert_eq!(resolved, cwd.canonicalize().unwrap());
    assert!(crate::commands::check::nuxt::is_nuxt_project_root(
        &resolved
    ));
    let _ = std::fs::remove_dir_all(&case_root);
}

#[test]
fn external_only_input_does_not_inherit_the_cwd_nuxt_project() {
    let case_root = unique_case_dir("external-only-nuxt-root");
    let _ = std::fs::remove_dir_all(&case_root);
    let cwd = case_root.join("app");
    let external_root = case_root.join("external");
    let external = external_root.join("Repro.vue");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&external_root).unwrap();
    std::fs::write(cwd.join("nuxt.config.ts"), "export default {}\n").unwrap();
    std::fs::write(&external, "<template />\n").unwrap();

    let resolved = resolve_program_nuxt_project_root(
        &cwd,
        &external_root,
        None,
        std::slice::from_ref(&external),
        &mut CanonicalPathCache::default(),
    );

    assert_eq!(resolved, external_root);
    assert!(!crate::commands::check::nuxt::is_nuxt_project_root(
        &resolved
    ));
    let _ = std::fs::remove_dir_all(&case_root);
}
