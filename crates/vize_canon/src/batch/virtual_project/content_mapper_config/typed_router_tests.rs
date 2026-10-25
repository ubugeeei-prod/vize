use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::ContentMapperProjectConfiguration;

fn host_configuration(
    path: &Path,
    compiler: Value,
) -> (ContentMapperProjectConfiguration, Option<PathBuf>) {
    ContentMapperProjectConfiguration::load_with_compiler_options(
        path,
        compiler.as_object().unwrap(),
    )
    .unwrap()
}

#[test]
fn host_root_is_admitted_once_and_explicit_plugin_root_takes_precedence() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"compilerOptions":{"rootDir":"authored"},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    ).unwrap();
    let admitted = host_configuration(&config, json!({"rootDir": root.join("host")}));
    assert_eq!(admitted.1.as_deref(), Some(root.join("host").as_path()));
    assert_eq!(
        admitted.0.watched_files,
        vec![config.to_string_lossy().into_owned()]
    );

    std::fs::write(
        &config,
        r#"{"vueCompilerOptions":{"plugins":[{"name":"vue-router/volar/sfc-typed-router","options":{"rootDir":"plugin"}}]}}"#,
    ).unwrap();
    let explicit = host_configuration(&config, json!({"rootDir": root.join("host")}));
    assert_eq!(explicit.1.as_deref(), Some(root.join("plugin").as_path()));
    std::fs::remove_file(&config).unwrap();
    assert_eq!(admitted.1.as_deref(), Some(root.join("host").as_path()));
    assert_eq!(explicit.1.as_deref(), Some(root.join("plugin").as_path()));
}

#[test]
fn enabled_identity_tracks_effective_root_but_disabled_identity_does_not() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    let config = root.join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    )
    .unwrap();
    let first = host_configuration(&config, json!({"rootDir": root.join("first")}));
    let second = host_configuration(&config, json!({"rootDir": root.join("second")}));
    assert_ne!(first.0.config_identity, second.0.config_identity);
    assert_eq!(
        host_configuration(&config, json!({})).1.as_deref(),
        Some(root.as_path())
    );

    std::fs::write(&config, r#"{"vueCompilerOptions":{"plugins":[]}}"#).unwrap();
    let disabled = host_configuration(&config, json!({"rootDir": root.join("first")}));
    let disabled_other = host_configuration(&config, json!({"rootDir": root.join("second")}));
    let legacy = ContentMapperProjectConfiguration::load(&config).unwrap();
    assert_eq!(disabled.1.as_deref(), None);
    assert_eq!(disabled_other.1.as_deref(), None);
    assert_eq!(disabled.0.config_identity, disabled_other.0.config_identity);
    assert_eq!(disabled.0.config_identity, legacy.config_identity);
    assert_ne!(first.0.config_identity, disabled.0.config_identity);
}

#[test]
fn inherited_opt_in_uses_selected_project_anchor_and_malformed_roots_decline() {
    let directory = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(directory.path());
    std::fs::write(
        root.join("base.json"),
        r#"{"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#,
    )
    .unwrap();
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"extends":"./base.json"}"#).unwrap();
    assert_eq!(
        host_configuration(&config, json!({"rootDir":"src"}))
            .1
            .as_deref(),
        Some(root.join("src").as_path())
    );
    assert_eq!(
        host_configuration(&config, json!({"rootDir":false}))
            .1
            .as_deref(),
        None
    );
    std::fs::write(&config, r#"{"vueCompilerOptions":{"plugins":[{"name":"vue-router/volar/sfc-typed-router","options":{"rootDir":false}}]}}"#).unwrap();
    assert_eq!(
        host_configuration(&config, json!({"rootDir":root}))
            .1
            .as_deref(),
        None
    );
}

#[test]
fn inferred_project_never_acquires_typed_router_authority() {
    let inferred = host_configuration(Path::new(""), json!({"rootDir":"/some/project"}));
    assert_eq!(inferred.1.as_deref(), None);
    assert_eq!(inferred.0.watched_files, Vec::<vize_l0::String>::new());
    assert_eq!(
        inferred.0.config_identity.as_str(),
        "vize-content-mapper-v1:7a38905296006c575d7d45d1d881690a3ee87361ed67ce64642e2504482da818"
    );
}
