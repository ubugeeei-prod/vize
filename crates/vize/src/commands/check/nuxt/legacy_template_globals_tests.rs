use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::cstr;

use super::{detect_legacy_nuxt_auto_imports, detect_nuxt_auto_imports};

#[test]
fn detects_legacy_nuxt2_fetch_state_and_route_template_globals() {
    let project_root = unique_case_dir("nuxt2-template-globals");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();

    let mut standard_options = VirtualTsOptions::default();
    let _ = detect_nuxt_auto_imports(&mut standard_options, &project_root);
    assert!(
        !standard_options
            .template_globals
            .iter()
            .any(|global| global.name == "$fetchState"),
        "standard Nuxt detection should not add Nuxt 2-only globals: {:#?}",
        standard_options.template_globals
    );

    let mut legacy_options = VirtualTsOptions::default();
    let _ = detect_legacy_nuxt_auto_imports(&mut legacy_options, &project_root);
    for expected in [
        "$config",
        "$fetchState",
        "$nuxt",
        "$route",
        "$router",
        "$store",
    ] {
        assert!(
            legacy_options
                .template_globals
                .iter()
                .any(|global| global.name == expected),
            "expected {expected} template global, got: {:#?}",
            legacy_options.template_globals
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}

fn unique_case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist");
    let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
    workspace_root
        .join("target")
        .join("vize-tests")
        .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
}
