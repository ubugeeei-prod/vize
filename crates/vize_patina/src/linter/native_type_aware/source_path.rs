//! Source-file path preparation for virtual type analysis.

use std::path::{Path, PathBuf};

/// Resolve the analyzed SFC filename to a complete source-file path.
///
/// Virtual TypeScript generation resolves relative imports against the parent
/// of this path, so callers must provide the file itself rather than its parent
/// directory.
pub(super) fn absolute_source_file(filename: &str) -> PathBuf {
    let path = Path::new(filename);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    std::env::current_dir().map_or_else(|_| path.to_path_buf(), |current| current.join(path))
}

#[cfg(test)]
mod tests {
    use super::absolute_source_file;
    use std::path::Path;

    #[test]
    fn relative_source_files_are_rooted_without_losing_the_filename() {
        let relative = Path::new("src/components/Button.vue");
        let resolved = absolute_source_file(relative.to_str().expect("UTF-8 fixture path"));

        assert!(resolved.is_absolute());
        assert!(resolved.ends_with(relative));
        assert_eq!(resolved.file_name(), relative.file_name());
    }

    #[test]
    fn absolute_source_files_are_preserved() {
        let absolute = std::env::current_dir()
            .expect("current directory")
            .join("src/components/Button.vue");

        assert_eq!(absolute_source_file(absolute.to_str().unwrap()), absolute);
    }
}
