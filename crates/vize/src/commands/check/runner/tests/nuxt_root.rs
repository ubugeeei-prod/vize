use super::super::resolve_nuxt_project_root;
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
