use std::path::Path;

use tower_lsp::lsp_types::Url;
use vize_l0::String;

use super::{NuxtTemplateNames, ServerState};

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
}

fn declarations(names: &[&str]) -> String {
    let mut source = String::from("export {}\ndeclare global {\n");
    for name in names {
        source.push_str("  const ");
        source.push_str(name);
        source.push_str(": () => void\n");
    }
    source.push_str("}\n");
    source
}

fn names(state: &ServerState, directory: &Path) -> Option<Vec<String>> {
    state
        .nuxt_template_names(directory)
        .map(|names| match names {
            NuxtTemplateNames::Generated(names) => names.to_vec(),
            NuxtTemplateNames::NotGenerated => vec!["<not generated>".into()],
        })
}

fn uri(path: &Path) -> Url {
    Url::from_file_path(path).unwrap()
}

#[test]
fn a_project_is_read_once_until_a_watched_event_names_its_declarations() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write(
        root,
        "nuxt.config.ts",
        "export default defineNuxtConfig({})\n",
    );
    write(
        root,
        ".nuxt/types/imports.d.ts",
        &declarations(&["formatCount"]),
    );
    let state = ServerState::new();
    let pages = root.join("app/pages");

    assert_eq!(names(&state, &pages), Some(vec!["formatCount".into()]));
    // Without an event the cached names stand, so a pass reads nothing.
    write(
        root,
        ".nuxt/types/imports.d.ts",
        &declarations(&["formatCount", "useCart"]),
    );
    assert_eq!(names(&state, &pages), Some(vec!["formatCount".into()]));

    assert!(!state.invalidate_nuxt_template_names([&uri(&root.join("app/pages/index.vue"))]));
    assert!(
        !state.invalidate_nuxt_template_names([&uri(&root.join("node_modules/nuxt/package.json"))])
    );
    assert_eq!(names(&state, &pages), Some(vec!["formatCount".into()]));

    assert!(state.invalidate_nuxt_template_names([&uri(&root.join(".nuxt/types/imports.d.ts"))]));
    assert_eq!(
        names(&state, &pages),
        Some(vec!["formatCount".into(), "useCart".into()])
    );
    assert_eq!(state.take_nuxt_notices(), Vec::<String>::new());
}

#[test]
fn a_missing_declaration_file_is_noticed_once_per_project() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path();
    write(
        root,
        "nuxt.config.ts",
        "export default defineNuxtConfig({})\n",
    );
    let state = ServerState::new();

    assert_eq!(
        names(&state, &root.join("app")),
        Some(vec!["<not generated>".into()])
    );
    assert_eq!(
        names(&state, &root.join("app/pages")),
        Some(vec!["<not generated>".into()])
    );
    let notice = vize_l0::cstr!(
        "Skipping vue/no-undefined-refs in {}: Nuxt has not generated {} yet, so auto-imported \
         names cannot be told from undefined ones. Run `nuxi prepare` to generate it.",
        root.display(),
        root.join(".nuxt/types/imports.d.ts").display()
    );
    assert_eq!(state.take_nuxt_notices(), [notice]);
    assert_eq!(state.take_nuxt_notices(), Vec::<String>::new());

    write(
        root,
        ".nuxt/types/imports.d.ts",
        &declarations(&["useCart"]),
    );
    assert!(state.invalidate_nuxt_template_names([&uri(&root.join(".nuxt/types/imports.d.ts"))]));
    assert_eq!(
        names(&state, &root.join("app")),
        Some(vec!["useCart".into()])
    );
    assert_eq!(state.take_nuxt_notices(), Vec::<String>::new());
}

#[test]
fn each_root_keeps_its_own_names_and_other_projects_read_nothing() {
    let workspace = tempfile::tempdir().unwrap();
    let root = workspace.path();
    write(root, "apps/shop/nuxt.config.ts", "export default {}\n");
    write(root, "apps/shop/package.json", "{}");
    write(
        root,
        "apps/shop/.nuxt/types/imports.d.ts",
        &declarations(&["useCart"]),
    );
    write(
        root,
        "apps/docs/nuxt.config.ts",
        "export default { buildDir: '.cache/nuxt' }\n",
    );
    write(root, "apps/docs/package.json", "{}");
    write(
        root,
        "apps/docs/.cache/nuxt/types/imports.d.ts",
        &declarations(&["useSidebar"]),
    );
    write(root, "apps/plain/package.json", "{}");
    // Declarations left behind without a Nuxt config are not Nuxt's.
    write(
        root,
        "apps/plain/.nuxt/types/imports.d.ts",
        &declarations(&["useCart"]),
    );
    let state = ServerState::new();

    assert_eq!(
        names(&state, &root.join("apps/shop/app")),
        Some(vec!["useCart".into()])
    );
    assert_eq!(
        names(&state, &root.join("apps/docs/app")),
        Some(vec!["useSidebar".into()])
    );
    assert_eq!(names(&state, &root.join("apps/plain/src")), None);
    assert!(state.invalidate_nuxt_template_names([&uri(&root.join("apps/docs/nuxt.config.ts"))]));
    assert_eq!(state.take_nuxt_notices(), Vec::<String>::new());
}
