//! `vize check` and the Content Mapper resolve `vueCompilerOptions` alike
//! (#11688). The mapper used to read only part of what `vize check` read, so
//! its hosts silently missed errors under `strictTemplates`.

use std::path::Path;

use super::vue_check_options::CHECK_OPTION_KEYS;
use super::{ContentMapperProjectConfiguration, VirtualProject};
use crate::virtual_ts::VirtualTsCheckOptions;

/// Both hosts' check options for one project configuration.
fn resolved(vue_compiler_options: &str) -> (VirtualTsCheckOptions, VirtualTsCheckOptions) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    std::fs::write(
        root.join("tsconfig.base.json"),
        vize_carton::cstr!(r#"{{ "vueCompilerOptions": {{ {vue_compiler_options} }} }}"#).as_str(),
    )
    .unwrap();
    // Through `extends`, as projects usually inherit these options.
    let path = root.join("tsconfig.json");
    std::fs::write(
        &path,
        r#"{ "extends": "./tsconfig.base.json", "compilerOptions": { "strict": true } }"#,
    )
    .unwrap();
    let mut project = VirtualProject::new(root).unwrap();
    project.set_tsconfig_path(Some(path.clone()));
    let mapper = ContentMapperProjectConfiguration::load(&path)
        .unwrap()
        .transform_options
        .virtual_ts_check_options();
    (project.virtual_ts_check_options, mapper)
}

#[test]
fn every_check_option_resolves_identically_in_both_hosts() {
    let (default_check, default_mapper) = resolved("");
    assert_eq!(default_check, default_mapper);
    for key in CHECK_OPTION_KEYS {
        for value in ["true", "false"] {
            let options = vize_carton::cstr!(r#""{key}": {value}"#);
            let (check, mapper) = resolved(&options);
            assert_eq!(check, mapper, "{options}");
            let strict = vize_carton::cstr!(r#""strictTemplates": true, {options}"#);
            let (check, mapper) = resolved(&strict);
            assert_eq!(check, mapper, "{strict}");
        }
        // Every listed key changes something, so none is listed but ignored.
        let (on, _) = resolved(&vize_carton::cstr!(r#""{key}": true"#));
        let (off, _) = resolved(&vize_carton::cstr!(r#""{key}": false"#));
        assert!(
            on != default_check || off != default_check,
            "{key} is ignored"
        );
    }
}

/// The shared resolver is the only reader of a check option, and it reads
/// only listed keys. A key added to one host alone fails here.
#[test]
fn check_options_are_read_only_by_the_shared_resolver() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/batch/virtual_project");
    let read = |path: &str| std::fs::read_to_string(directory.join(path)).unwrap();
    for host in [
        "tsconfig_gen/vue_compiler_options.rs",
        "content_mapper_config.rs",
    ] {
        let source = read(host);
        let leaked: Vec<&str> = CHECK_OPTION_KEYS
            .into_iter()
            .filter(|key| source.contains(&*vize_carton::cstr!("\"{key}\"")))
            .collect();
        assert_eq!(
            leaked,
            Vec::<&str>::new(),
            "{host} reads check keys outside vue_check_options::resolve"
        );
    }
    let resolver = read("vue_check_options.rs");
    let body = resolver.split_once("pub(super) fn resolve(").unwrap().1;
    // Every quoted literal in the resolver body is a key it reads.
    let mut reads: Vec<&str> = body.split('"').skip(1).step_by(2).collect();
    reads.sort_unstable();
    reads.dedup();
    let mut listed = CHECK_OPTION_KEYS.to_vec();
    listed.sort_unstable();
    assert_eq!(reads, listed, "resolve reads exactly the listed keys");
}
