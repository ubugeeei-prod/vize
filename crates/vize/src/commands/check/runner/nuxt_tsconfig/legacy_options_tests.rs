use std::{path::Path, sync::atomic::AtomicUsize};

use super::write_nuxt_fallback_tsconfig;
use crate::commands::check::nuxt::NuxtPathAlias;

fn case_dir(name: &str) -> std::path::PathBuf {
    static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);
    let case_id = NEXT_CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target/vize-tests")
        .join(format!(
            "nuxt-fallback-options-{name}-{}-{case_id}",
            std::process::id()
        ))
}

#[test]
fn inherited_config_uses_the_typescript_compatibility_floor() {
    let root = case_dir("inherited");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let tsconfig = root.join("tsconfig.json");
    std::fs::write(
        &tsconfig,
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Node",
    "baseUrl": "."
  }
}
"#,
    )
    .unwrap();

    let wrapper = write_nuxt_fallback_tsconfig(
        Some(&tsconfig),
        &root,
        &root,
        &[NuxtPathAlias {
            pattern: "~/*".into(),
            targets: vec!["./*".into()],
        }],
    )
    .unwrap();
    let wrapper: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(wrapper).unwrap()).unwrap();

    assert_eq!(
        wrapper["compilerOptions"]["ignoreDeprecations"],
        serde_json::json!("6.0")
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn wrapper_without_an_inherited_config_adds_no_compatibility_option() {
    let root = case_dir("standalone");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    let wrapper = write_nuxt_fallback_tsconfig(
        None,
        &root,
        &root,
        &[NuxtPathAlias {
            pattern: "~/*".into(),
            targets: vec!["./*".into()],
        }],
    )
    .unwrap();
    let wrapper: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(wrapper).unwrap()).unwrap();

    assert!(
        wrapper["compilerOptions"]
            .get("ignoreDeprecations")
            .is_none()
    );

    let _ = std::fs::remove_dir_all(&root);
}
