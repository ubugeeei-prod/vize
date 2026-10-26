//! Real external declaration inputs for the literal reference regression.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(
    clippy::disallowed_types,
    reason = "fixture strings retain exact bytes"
)]

use super::fix_history_paths::safe_relative;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use vize_canon::virtual_ts::VirtualTsOptions;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceOptions {
    directory: String,
}

pub(crate) fn project_root(owned: &Path) -> PathBuf {
    let root = owned.join("project");
    std::fs::create_dir(&root).expect("the checked project must be created");
    std::fs::canonicalize(root).expect("the checked project must resolve")
}

pub(crate) fn options(project: &Path) -> VirtualTsOptions {
    let config: ReferenceOptions = serde_json::from_slice(
        &std::fs::read(project.join("reference-options.json"))
            .expect("reference options must exist"),
    )
    .expect("reference options must keep their registered schema");
    let directory = safe_relative(&config.directory);
    assert_eq!(directory.file_name(), Some(directory.as_os_str()));
    assert_ne!(directory.as_path(), Path::new("project"));
    let target = project
        .parent()
        .expect("the owned outer directory must exist")
        .join(directory);
    std::fs::create_dir(&target).expect("the external declaration directory must be fresh");
    let source = std::fs::read(project.join("fixture-inputs/ambient.d.ts.txt"))
        .expect("the pinned declaration source must exist");
    let declaration = target.join("ambient.d.ts");
    std::fs::write(&declaration, &source).expect("the real external declaration must materialize");
    assert_eq!(
        std::fs::read(&declaration).expect("the declaration must remain readable"),
        source
    );
    VirtualTsOptions {
        reference_paths: vec![declaration.to_string_lossy().as_ref().into()],
        ..Default::default()
    }
}
