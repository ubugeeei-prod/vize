use std::path::{Path, PathBuf};

pub(super) fn find_vscode_vsix() -> Option<PathBuf> {
    find_vscode_vsix_in_locations([
        PathBuf::from("editors/vscode"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("../../editors/vscode")))
            .unwrap_or_default(),
    ])
}

pub(super) fn find_vscode_vsix_in_locations(
    locations: impl IntoIterator<Item = PathBuf>,
) -> Option<PathBuf> {
    for base in locations {
        if let Ok(entries) = std::fs::read_dir(base) {
            let mut candidates = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| is_vsix_file(path))
                .collect::<Vec<_>>();
            candidates.sort();

            if let Some(path) = candidates.into_iter().next() {
                return Some(path);
            }
        }
    }

    None
}

pub(super) fn is_vsix_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("vsix"))
}

pub(super) fn find_zed_extension_source() -> Option<PathBuf> {
    find_zed_extension_source_in_locations([
        PathBuf::from("editors/zed"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("../../editors/zed")))
            .unwrap_or_default(),
    ])
}

pub(super) fn find_zed_extension_source_in_locations(
    locations: impl IntoIterator<Item = PathBuf>,
) -> Option<PathBuf> {
    locations
        .into_iter()
        .find(|path| path.join("extension.toml").exists())
}

pub(super) fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&from, &to)?;
        } else if file_type.is_file() {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        copy_dir_all, find_vscode_vsix_in_locations, find_zed_extension_source_in_locations,
        is_vsix_file,
    };

    #[test]
    fn vsix_file_detection_accepts_files_case_insensitively() {
        let dir = tempfile::tempdir().unwrap();
        let vsix = dir.path().join("vize.VSIX");
        std::fs::write(&vsix, "package").unwrap();

        assert!(is_vsix_file(&vsix));
    }

    #[test]
    fn vsix_file_detection_rejects_directories_and_other_extensions() {
        let dir = tempfile::tempdir().unwrap();
        let fake_dir = dir.path().join("fake.vsix");
        let readme = dir.path().join("README.md");
        std::fs::create_dir(&fake_dir).unwrap();
        std::fs::write(&readme, "not a vsix").unwrap();

        assert!(!is_vsix_file(&fake_dir));
        assert!(!is_vsix_file(&readme));
    }

    #[test]
    fn vscode_vsix_lookup_is_deterministic_within_first_matching_location() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let beta = first.path().join("vize-beta.vsix");
        let alpha = first.path().join("vize-alpha.vsix");
        let fallback = second.path().join("vize-fallback.vsix");
        std::fs::write(&beta, "beta").unwrap();
        std::fs::write(&alpha, "alpha").unwrap();
        std::fs::write(&fallback, "fallback").unwrap();

        let found = find_vscode_vsix_in_locations([
            first.path().to_path_buf(),
            second.path().to_path_buf(),
        ]);

        assert_eq!(found.as_deref(), Some(alpha.as_path()));
    }

    #[test]
    fn vscode_vsix_lookup_skips_missing_locations_until_a_package_exists() {
        let dir = tempfile::tempdir().unwrap();
        let package = dir.path().join("vize.vsix");
        std::fs::write(&package, "package").unwrap();

        let found =
            find_vscode_vsix_in_locations([dir.path().join("missing"), dir.path().to_path_buf()]);

        assert_eq!(found.as_deref(), Some(package.as_path()));
    }

    #[test]
    fn zed_source_lookup_requires_extension_manifest() {
        let missing_manifest = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        std::fs::write(source.path().join("extension.toml"), "id = \"vize\"").unwrap();

        let found = find_zed_extension_source_in_locations([
            missing_manifest.path().to_path_buf(),
            source.path().to_path_buf(),
        ]);

        assert_eq!(found.as_deref(), Some(source.path()));
    }

    #[test]
    fn copy_dir_all_copies_nested_files_without_touching_source() {
        let source = tempfile::tempdir().unwrap();
        let nested = source.path().join("grammars");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(source.path().join("extension.toml"), "id = \"vize\"").unwrap();
        std::fs::write(nested.join("vue.scm"), "(source_file)").unwrap();

        let target = tempfile::tempdir().unwrap();
        let install_dir = target.path().join("vize");
        copy_dir_all(source.path(), &install_dir).unwrap();

        assert_eq!(
            std::fs::read_to_string(install_dir.join("extension.toml")).unwrap(),
            "id = \"vize\""
        );
        assert_eq!(
            std::fs::read_to_string(install_dir.join("grammars/vue.scm")).unwrap(),
            "(source_file)"
        );
        assert!(source.path().join("extension.toml").exists());
    }
}
