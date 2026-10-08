use std::path::Path;

use tower_lsp::lsp_types::Url;
use vize_canon::virtual_ts::{TemplateGlobal, VirtualTsOptions};
use vize_l0::cstr;

use super::ServerState;

const DECLARATION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-reference-options/components.d.ts.txt"
));
const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/lsp/editor-reference-options/vize.config.json.txt"
));

fn options(state: &ServerState) -> VirtualTsOptions {
    crate::runtime::block_on(state.editor_virtual_ts_options())
}

fn expected(paths: &[&Path]) -> VirtualTsOptions {
    VirtualTsOptions {
        template_globals: vec![TemplateGlobal {
            name: "currentRoute".into(),
            type_annotation: "{ path: string }".into(),
            default_value: "{ path: '/' }".into(),
        }],
        css_modules: Vec::new(),
        auto_import_stubs: Vec::new(),
        auto_import_bindings: Vec::new(),
        external_template_bindings: Vec::new(),
        reference_paths: paths
            .iter()
            .map(|path| {
                std::fs::canonicalize(path)
                    .unwrap()
                    .to_string_lossy()
                    .as_ref()
                    .into()
            })
            .collect(),
        strict_instance_globals: false,
    }
}

fn configured_state(root: &Path) -> ServerState {
    std::fs::write(root.join("vize.config.json"), CONFIG).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());
    state.load_workspace_config(root);
    state
}

fn assert_options(state: &ServerState, expected: &VirtualTsOptions) {
    // Compare every option and the complete ordered declaration vector.
    assert_eq!(cstr!("{:?}", options(state)), cstr!("{expected:?}"));
}

#[test]
fn editor_options_preserve_config_and_all_workspace_declaration_kinds() {
    let root = tempfile::tempdir().unwrap();
    let nuxt = root.path().join(".nuxt");
    let dependency = root.path().join("node_modules/pkg");
    std::fs::create_dir_all(&nuxt).unwrap();
    std::fs::create_dir_all(&dependency).unwrap();
    let components = nuxt.join("components.d.cts");
    let imports = nuxt.join("imports.d.ts");
    let virtual_imports = nuxt.join("virtual-imports.d.mts");
    for path in [&components, &imports, &virtual_imports] {
        std::fs::write(path, DECLARATION).unwrap();
    }
    std::fs::write(dependency.join("global.d.ts"), DECLARATION).unwrap();
    let state = configured_state(root.path());
    let expected = expected(&[&components, &imports, &virtual_imports]);

    assert_options(&state, &expected);
    assert_options(&state, &expected);
}

#[test]
fn editor_options_refresh_create_rename_delete_and_workspace_changes() {
    let root = tempfile::tempdir().unwrap();
    let nuxt = root.path().join(".nuxt");
    std::fs::create_dir(&nuxt).unwrap();
    let state = configured_state(root.path());
    assert_options(&state, &expected(&[]));

    let created = nuxt.join("components.d.ts");
    std::fs::write(&created, DECLARATION).unwrap();
    // Repeated requests keep the existing discovery generation.
    assert_options(&state, &expected(&[]));
    let created_uri = Url::from_file_path(&created).unwrap();
    assert!(state.invalidate_global_component_references([created_uri.as_str()]));
    assert_options(&state, &expected(&[&created]));

    let renamed = nuxt.join("components.d.mts");
    std::fs::rename(&created, &renamed).unwrap();
    let renamed_uri = Url::from_file_path(&renamed).unwrap();
    assert!(
        state.invalidate_global_component_references([created_uri.as_str(), renamed_uri.as_str()])
    );
    assert_options(&state, &expected(&[&renamed]));

    std::fs::remove_file(&renamed).unwrap();
    assert!(state.invalidate_global_component_references([renamed_uri.as_str()]));
    assert_options(&state, &expected(&[]));

    let next_root = tempfile::tempdir().unwrap();
    let next = next_root.path().join("globals.d.cts");
    std::fs::write(&next, DECLARATION).unwrap();
    state.set_workspace_root(next_root.path().to_path_buf());
    assert_options(&state, &expected(&[&next]));
}

#[test]
fn editor_options_reload_template_globals_without_losing_declarations() {
    let root = tempfile::tempdir().unwrap();
    let declaration = root.path().join("globals.d.ts");
    std::fs::write(&declaration, DECLARATION).unwrap();
    let state = configured_state(root.path());
    assert_options(&state, &expected(&[&declaration]));

    std::fs::write(root.path().join("vize.config.json"), "{}").unwrap();
    state.load_workspace_config(root.path());
    let mut expected = expected(&[&declaration]);
    expected.template_globals.clear();
    assert_options(&state, &expected);
}
