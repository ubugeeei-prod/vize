use super::{VirtualProject, unique_case_dir};
use std::fs;

#[test]
fn test_virtual_project_new() {
    let case_dir = unique_case_dir("new");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(&case_dir).unwrap();

    let project = VirtualProject::new(&case_dir).unwrap();
    assert_eq!(project.project_root(), case_dir.as_path());
    assert!(project.virtual_root().ends_with("node_modules/.vize/canon"));

    let _ = fs::remove_dir_all(&case_dir);
}

#[cfg(unix)]
#[test]
fn virtual_root_resolves_a_symlinked_node_modules_to_its_real_location() {
    // pnpm-style stores, monorepo hoisting shims, worktree lanes, and
    // containers that bind-mount dependencies all reach `node_modules` through
    // a symlink. Composing the virtual root by joining onto the canonical
    // project root leaves a path that traverses that link, so the virtual files
    // land outside the root, Corsa reports diagnostics under the link target,
    // and mapping them back to the authored SFC fails — silently, which is the
    // dangerous direction.
    let case_dir = unique_case_dir("symlinked-node-modules");
    let _ = fs::remove_dir_all(&case_dir);
    let project_root = case_dir.join("project");
    let store = case_dir.join("store");
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(&store).unwrap();
    std::os::unix::fs::symlink(&store, project_root.join("node_modules")).unwrap();

    let project = VirtualProject::new(&project_root).unwrap();

    let real_store = vize_carton::path::canonicalize_non_verbatim(&store);
    assert_eq!(
        project.virtual_root(),
        real_store.join(".vize").join("canon").as_path(),
        "virtual root must be expressed in real-path terms, not through the link"
    );
    assert!(
        !project.virtual_root().starts_with(project.project_root()),
        "a symlinked store legitimately sits outside the project root; the \
         virtual root must say so rather than pretending to be inside it"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[cfg(unix)]
#[test]
fn dependencies_stay_resolvable_when_node_modules_is_a_symlinked_store() {
    // Asserting the virtual root's path alone would still pass if the real-path
    // root broke module resolution: a store named something other than
    // `node_modules` is not a lookup directory for TypeScript's ancestor walk,
    // so a bare `vue` import could no longer resolve through the store's
    // parents. Materialization is what makes that a non-issue — the virtual
    // root carries its own `node_modules`, populated from packages resolved
    // against the project root — and this pins it for the symlinked layout.
    let case_dir = unique_case_dir("symlinked-node-modules-deps");
    let _ = fs::remove_dir_all(&case_dir);
    let project_root = case_dir.join("project");
    let store = case_dir.join("store");
    fs::create_dir_all(&project_root).unwrap();
    fs::create_dir_all(store.join("vue")).unwrap();
    fs::write(
        store.join("vue").join("package.json"),
        "{\"name\":\"vue\",\"version\":\"3.5.0\",\"types\":\"index.d.ts\"}",
    )
    .unwrap();
    fs::write(
        store.join("vue").join("index.d.ts"),
        "export declare function defineComponent(options: unknown): unknown;\n",
    )
    .unwrap();
    std::os::unix::fs::symlink(&store, project_root.join("node_modules")).unwrap();

    let project = VirtualProject::new(&project_root).unwrap();
    project.materialize().unwrap();

    let virtual_vue = project.virtual_root().join("node_modules").join("vue");
    assert!(
        virtual_vue.join("package.json").is_file(),
        "a bare `vue` import from a virtual file must resolve in the virtual \
         root's own `node_modules`, not by walking the store's parents: {}",
        virtual_vue.display()
    );
    assert!(
        virtual_vue.starts_with(vize_carton::path::canonicalize_non_verbatim(&store)),
        "the lookup root must sit inside the real-path virtual root"
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn virtual_root_stays_inside_the_project_for_a_real_node_modules_directory() {
    let case_dir = unique_case_dir("real-node-modules");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("node_modules")).unwrap();

    let project = VirtualProject::new(&case_dir).unwrap();

    assert_eq!(
        project.virtual_root(),
        case_dir.join("node_modules").join(".vize").join("canon"),
    );

    let _ = fs::remove_dir_all(&case_dir);
}
