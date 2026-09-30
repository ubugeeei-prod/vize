//! Filesystem path helpers shared by CLI and native type-checking code.

use std::path::{Path, PathBuf};

/// Canonicalize a path while avoiding Windows extended-length prefixes in the
/// returned path.
///
/// `std::fs::canonicalize` returns paths such as `\\?\C:\repo` on Windows.
/// Those are valid Win32 paths, but Node/TypeScript normalize them to
/// `//?/C:/repo` and can reject them as missing config paths. The checker only
/// needs a stable absolute path, so strip the verbatim prefix back to the
/// ordinary DOS/UNC spelling before handing paths to TypeScript-facing code.
pub fn canonicalize_non_verbatim(path: &Path) -> PathBuf {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    normalize_windows_verbatim_path(canonical)
}

/// Strip a Windows verbatim path prefix from `path` when running on Windows.
pub fn normalize_windows_verbatim_path(path: PathBuf) -> PathBuf {
    normalize_windows_verbatim_path_impl(path)
}

#[cfg(windows)]
fn normalize_windows_verbatim_path_impl(path: PathBuf) -> PathBuf {
    let Some(path_str) = path.to_str() else {
        return path;
    };
    strip_windows_verbatim_prefix(path_str)
        .map(PathBuf::from)
        .unwrap_or(path)
}

#[cfg(not(windows))]
fn normalize_windows_verbatim_path_impl(path: PathBuf) -> PathBuf {
    path
}

#[cfg(any(windows, test))]
fn strip_windows_verbatim_prefix(value: &str) -> Option<std::string::String> {
    if let Some(rest) = value.strip_prefix("\\\\?\\UNC\\") {
        let mut normalized = std::string::String::with_capacity(rest.len() + 2);
        normalized.push_str("\\\\");
        normalized.push_str(rest);
        return Some(normalized);
    }
    if let Some(rest) = value.strip_prefix("//?/UNC/") {
        let mut normalized = std::string::String::with_capacity(rest.len() + 2);
        normalized.push_str("//");
        normalized.push_str(rest);
        return Some(normalized);
    }
    if let Some(rest) = value.strip_prefix("\\\\?\\") {
        return Some(rest.to_owned());
    }
    if let Some(rest) = value.strip_prefix("//?/") {
        return Some(rest.to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::strip_windows_verbatim_prefix;

    #[test]
    fn strips_windows_drive_verbatim_prefixes() {
        assert_eq!(
            strip_windows_verbatim_prefix("\\\\?\\D:\\a\\ox-jsdoc\\ox-jsdoc\\apps\\playground")
                .as_deref(),
            Some("D:\\a\\ox-jsdoc\\ox-jsdoc\\apps\\playground")
        );
    }

    #[test]
    fn strips_windows_unc_verbatim_prefixes() {
        assert_eq!(
            strip_windows_verbatim_prefix("\\\\?\\UNC\\server\\share\\project").as_deref(),
            Some("\\\\server\\share\\project")
        );
    }

    #[test]
    fn strips_forward_slash_verbatim_prefixes_from_tool_output() {
        assert_eq!(
            strip_windows_verbatim_prefix("//?/D:/repo/node_modules/.vize/canon/tsconfig.json")
                .as_deref(),
            Some("D:/repo/node_modules/.vize/canon/tsconfig.json")
        );
    }

    #[test]
    fn leaves_normal_paths_alone() {
        assert_eq!(strip_windows_verbatim_prefix("D:\\repo\\src"), None);
        assert_eq!(strip_windows_verbatim_prefix("/repo/src"), None);
    }
}
