use std::path::{Path, PathBuf};

pub(super) fn find_vscode_vsix() -> Option<PathBuf> {
    let repo_source = PathBuf::from("editors/vscode");
    let exe_source = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("../../editors/vscode")))
        .unwrap_or_default();

    find_vscode_vsix_in_locations([
        repo_source.join("dist"),
        repo_source,
        exe_source.join("dist"),
        exe_source,
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
        .find(|path| path.join("extension.toml").is_file())
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
    fn vscode_vsix_lookup_prefers_packaged_dist_directory() {
        let extension_dir = tempfile::tempdir().unwrap();
        let dist_dir = extension_dir.path().join("dist");
        std::fs::create_dir(&dist_dir).unwrap();
        let stale_package = extension_dir.path().join("vize-old.vsix");
        let packaged = dist_dir.join("vize.vsix");
        std::fs::write(&stale_package, "old").unwrap();
        std::fs::write(&packaged, "new").unwrap();

        let found = find_vscode_vsix_in_locations([
            extension_dir.path().join("dist"),
            extension_dir.path().to_path_buf(),
        ]);

        assert_eq!(found.as_deref(), Some(packaged.as_path()));
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
    fn zed_source_lookup_rejects_directory_named_extension_manifest() {
        let fake_source = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        std::fs::create_dir(fake_source.path().join("extension.toml")).unwrap();
        std::fs::write(source.path().join("extension.toml"), "id = \"vize\"").unwrap();

        let found = find_zed_extension_source_in_locations([
            fake_source.path().to_path_buf(),
            source.path().to_path_buf(),
        ]);

        assert_eq!(found.as_deref(), Some(source.path()));
    }

    #[test]
    fn zed_source_lookup_prefers_first_real_manifest() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        std::fs::write(first.path().join("extension.toml"), "id = \"first\"").unwrap();
        std::fs::write(second.path().join("extension.toml"), "id = \"second\"").unwrap();

        let found = find_zed_extension_source_in_locations([
            first.path().to_path_buf(),
            second.path().to_path_buf(),
        ]);

        assert_eq!(found.as_deref(), Some(first.path()));
    }

    #[test]
    fn vscode_vsix_lookup_rejects_nested_or_suffix_matches() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("vize.vsix"), "nested").unwrap();
        std::fs::write(dir.path().join("vize.vsix.bak"), "backup").unwrap();

        let found = find_vscode_vsix_in_locations([dir.path().to_path_buf()]);

        assert!(found.is_none());
    }

    #[test]
    fn vsix_file_detection_rejects_missing_and_suffix_extensions() {
        let dir = tempfile::tempdir().unwrap();
        let no_extension = dir.path().join("vize");
        let suffix = dir.path().join("vize.vsix.bak");
        std::fs::write(&no_extension, "package").unwrap();
        std::fs::write(&suffix, "package").unwrap();

        assert!(!is_vsix_file(&no_extension));
        assert!(!is_vsix_file(&suffix));
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

    #[test]
    fn copy_dir_all_overwrites_files_and_keeps_unrelated_targets() {
        let source = tempfile::tempdir().unwrap();
        std::fs::write(source.path().join("extension.toml"), "id = \"new\"").unwrap();

        let target = tempfile::tempdir().unwrap();
        let install_dir = target.path().join("vize");
        std::fs::create_dir(&install_dir).unwrap();
        std::fs::write(install_dir.join("extension.toml"), "id = \"old\"").unwrap();
        std::fs::write(install_dir.join("local-only.txt"), "keep").unwrap();

        copy_dir_all(source.path(), &install_dir).unwrap();

        assert_eq!(
            std::fs::read_to_string(install_dir.join("extension.toml")).unwrap(),
            "id = \"new\""
        );
        assert_eq!(
            std::fs::read_to_string(install_dir.join("local-only.txt")).unwrap(),
            "keep"
        );
    }

    #[test]
    fn copy_dir_all_returns_error_for_missing_source() {
        let target = tempfile::tempdir().unwrap();
        let missing = target.path().join("missing-source");
        let install_dir = target.path().join("vize");

        let error = copy_dir_all(&missing, &install_dir).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    }
}
