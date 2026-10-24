//! Bounded regular-byte native distribution custody.

use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
};

pub(crate) struct RuntimeDistribution {
    pub(crate) executable: PathBuf,
    original: PathBuf,
    pub(super) copied: PathBuf,
    names: Vec<OsString>,
    before: BTreeMap<String, String>,
    _relay: Option<tempfile::TempDir>,
}

impl RuntimeDistribution {
    pub(crate) fn copy(runtime: &Path, project: &Path, relay: bool) -> Self {
        let native = runtime.canonicalize().unwrap();
        let original = native.parent().unwrap().to_path_buf();
        assert!(
            original.join("lib.dom.d.ts").is_file(),
            "physical native distribution required"
        );
        let mut entries = std::fs::read_dir(&original)
            .unwrap()
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        entries.sort_by_key(std::fs::DirEntry::file_name);
        let mut bytes = 0;
        let names = entries
            .iter()
            .map(|entry| {
                assert!(
                    entry.file_type().unwrap().is_file(),
                    "self-contained regular distribution"
                );
                bytes += entry.metadata().unwrap().len();
                entry.file_name()
            })
            .collect::<Vec<_>>();
        assert!(bytes < 40 * 1024 * 1024, "bounded native distribution copy");
        let copied = project.join("runtime-distribution");
        std::fs::create_dir_all(&copied).unwrap();
        for entry in entries {
            let target = copied.join(entry.file_name());
            assert!(
                !target.exists(),
                "native copy must preserve authored sibling files"
            );
            std::fs::copy(entry.path(), &target).unwrap();
            assert!(!target.symlink_metadata().unwrap().file_type().is_symlink());
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let source = entry.metadata().unwrap();
                let copy = target.metadata().unwrap();
                assert_eq!(copy.nlink(), 1);
                assert_ne!((source.dev(), source.ino()), (copy.dev(), copy.ino()));
            }
        }
        let physical = copied.canonicalize().unwrap();
        assert_eq!(
            physical,
            project.canonicalize().unwrap().join("runtime-distribution")
        );
        assert!(
            !physical
                .components()
                .any(|part| part.as_os_str() == "node_modules")
        );
        let before = snapshot(&original, &names);
        assert_eq!(snapshot(&copied, &names), before);
        let executable = copied.join(native.file_name().unwrap());
        let (executable, relay_owner) = if relay {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                // The configured relay is physically OUTSIDE the writable root;
                // its actual native executable and libraries remain INSIDE it.
                let owner = tempfile::tempdir_in(project.parent().unwrap()).unwrap();
                let wrapper = owner.path().join("native-relay");
                let quoted = executable.to_str().unwrap().replace('\'', "'\\''");
                std::fs::write(&wrapper, format!("#!/bin/sh\nexec '{quoted}' \"$@\"\n")).unwrap();
                std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
                let physical_owner = owner.path().canonicalize().unwrap();
                let physical_project = project.canonicalize().unwrap();
                assert_eq!(
                    wrapper.canonicalize().unwrap(),
                    physical_owner.join("native-relay")
                );
                assert_eq!(physical_owner.parent(), physical_project.parent());
                assert_ne!(physical_owner, physical_project);
                (wrapper, Some(owner))
            }
            #[cfg(not(unix))]
            panic!("shell relay controls are Unix only");
        } else {
            (executable, None)
        };
        println!(
            "{}",
            json!({
                "kind":"actual-physical-native-distribution-before",
                "configuredExecutable":executable,"original":original,"copied":physical,
                "files":names.len(),"bytes":bytes,"sha256":before,
                "copy":"regular byte copies; no native symlinks or hardlinks",
                "relayOutsideWritableRoot":relay
            })
        );
        Self {
            executable,
            original,
            copied,
            names,
            before,
            _relay: relay_owner,
        }
    }

    pub(crate) fn assert_unchanged(&self) {
        let original_after = snapshot(&self.original, &self.names);
        let copied_after = snapshot(&self.copied, &self.names);
        assert_eq!(
            original_after, self.before,
            "all original native assets must be unchanged"
        );
        assert_eq!(
            copied_after, self.before,
            "all copied native assets must be unchanged"
        );
        println!(
            "{}",
            json!({
                "kind":"actual-physical-native-distribution-after",
                "originalSha256":original_after,"copiedSha256":copied_after,
                "nativeEditsApplied":false,"allAssetHashesUnchanged":true
            })
        );
    }
}

fn snapshot(root: &Path, names: &[OsString]) -> BTreeMap<String, String> {
    names
        .iter()
        .map(|name| {
            let path = root.join(name);
            assert!(!path.symlink_metadata().unwrap().file_type().is_symlink());
            (
                name.to_str().unwrap().to_owned(),
                Sha256::digest(std::fs::read(path).unwrap())
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect(),
            )
        })
        .collect()
}
