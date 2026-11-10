//! Filesystem path helpers shared by CLI and native type-checking code.

use std::path::{Path, PathBuf};

/// Git metadata is never a product source tree, even for an explicit glob.
/// Match whole components so `.github`, `.gitignore` and `foo.git` remain
/// ordinary paths. This lexical check is safe on every walker entry.
#[inline]
pub fn is_git_metadata_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == ".git")
}

/// Canonicalize a path while avoiding Windows extended-length prefixes in the
/// returned path.
///
/// `std::fs::canonicalize` returns paths such as `\\?\C:\repo` on Windows.
/// Those are valid Win32 paths, but Node/TypeScript normalize them to
/// `//?/C:/repo` and can reject them as missing config paths. The checker only
/// needs a stable absolute path, so strip the verbatim prefix back to the
/// ordinary DOS/UNC spelling before handing paths to TypeScript-facing code.
pub fn canonicalize_non_verbatim(path: &Path) -> PathBuf {
    crate::micro_count!("fs.canonicalize.calls", 1);
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    normalize_windows_verbatim_path(canonical)
}

/// The physical spelling of `path`: its deepest existing ancestor is
/// canonicalized (resolving symlinks) and the rest is kept, so unsaved or newly
/// renamed documents still map. Falls back to `path` when nothing exists.
pub fn physical_path(path: &Path) -> PathBuf {
    for ancestor in path.ancestors() {
        crate::micro_count!("fs.canonicalize.calls", 1);
        if let Ok(physical) = ancestor.canonicalize()
            && let Ok(relative) = path.strip_prefix(ancestor)
        {
            return normalize_windows_verbatim_path(if relative.as_os_str().is_empty() {
                physical
            } else {
                physical.join(relative)
            });
        }
    }
    path.to_path_buf()
}

/// `sort(); dedup();` for path lists dominated by repeats (resolver inputs).
///
/// `PathBuf` ordering walks components, so sorting every repeat is slow. Byte
/// copies are dropped first with a byte sort, then only unique paths take the
/// component-ordered sort, giving the same order and the same set under
/// `Path` equality.
pub fn sort_dedup_paths(paths: &mut Vec<PathBuf>) {
    paths.sort_unstable_by(|left, right| left.as_os_str().cmp(right.as_os_str()));
    paths.dedup_by(|left, right| left.as_os_str() == right.as_os_str());
    paths.sort();
    paths.dedup();
}

/// Strip a Windows verbatim path prefix from `path` when running on Windows
/// and the plain spelling names the same file.
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

/// [`normalize_windows_verbatim_path`] for a UTF-8 path string, such as a
/// compiler dependency handed to a host file watcher.
pub fn normalize_windows_verbatim_str(path: &str) -> std::borrow::Cow<'_, str> {
    #[cfg(windows)]
    if let Some(plain) = strip_windows_verbatim_prefix(path) {
        return std::borrow::Cow::Owned(plain);
    }
    std::borrow::Cow::Borrowed(path)
}

/// `\\?\C:\x` → `C:\x` and `\\?\UNC\host\share\x` → `\\host\share\x` (also
/// the `//?/` spelling tools print), only when the plain spelling names the
/// same file, as `dunce::simplified` decides. Length is not limited: Rust std
/// and Node re-add the prefix for long paths themselves.
#[cfg(any(windows, test))]
#[expect(
    clippy::disallowed_types,
    reason = "Preserve Windows path prefix normalization and its existing owned string"
)]
fn strip_windows_verbatim_prefix(value: &str) -> Option<std::string::String> {
    let (rest, separator) = value
        .strip_prefix("\\\\?\\")
        .map(|rest| (rest, '\\'))
        .or_else(|| value.strip_prefix("//?/").map(|rest| (rest, '/')))?;
    let mut components = rest.split(separator);
    let head = components.next()?;
    let mut plain = std::string::String::with_capacity(rest.len() + 2);
    if head.eq_ignore_ascii_case("UNC") {
        let server = components.next()?;
        let share = components.next()?;
        if !win32_component(server) || !win32_component(share) {
            return None;
        }
        plain.push(separator);
        plain.push(separator);
        plain.push_str(rest.get(4..)?);
    } else {
        let [drive, b':'] = head.as_bytes() else {
            return None;
        };
        // A bare `C:` is drive-relative, so it keeps its verbatim spelling.
        if !drive.is_ascii_alphabetic() || rest.len() == head.len() {
            return None;
        }
        plain.push_str(rest);
    }
    components
        .all(|component| component.is_empty() || win32_component(component))
        .then_some(plain)
}

/// A name Win32 path parsing keeps as written: verbatim paths take `..`,
/// trailing dots and spaces, and DOS device names literally.
#[cfg(any(windows, test))]
fn win32_component(name: &str) -> bool {
    const DEVICES: [&str; 32] = [
        "CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$", "COM0", "COM1", "COM2", "COM3", "COM4",
        "COM5", "COM6", "COM7", "COM8", "COM9", "COM¹", "COM²", "COM³", "LPT0", "LPT1", "LPT2",
        "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "LPT¹", "LPT²", "LPT³",
    ];
    let stem = name
        .split_once('.')
        .map_or(name, |(stem, _)| stem)
        .trim_end_matches(' ');
    !name.is_empty()
        && name.encode_utf16().count() <= 255
        && !name.ends_with(['.', ' '])
        && !name.chars().any(|character| {
            matches!(
                character,
                '\0'..='\x1f' | '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            )
        })
        && !DEVICES
            .iter()
            .any(|device| stem.eq_ignore_ascii_case(device))
}

#[cfg(test)]
mod tests {
    use super::{sort_dedup_paths, strip_windows_verbatim_prefix};
    use std::path::PathBuf;

    #[test]
    fn sort_dedup_paths_matches_component_sort_and_dedup() {
        let input = [
            "/r/a-b", "/r/a/b", "/r/a", "/r/a/b", "/r/a-b", "/r/a.b", "/r/a/b",
        ];
        let mut expected: Vec<PathBuf> = input.iter().map(PathBuf::from).collect();
        expected.sort();
        expected.dedup();
        let mut actual: Vec<PathBuf> = input.iter().map(PathBuf::from).collect();
        sort_dedup_paths(&mut actual);
        assert_eq!(actual, expected);
        assert_eq!(actual.first(), Some(&PathBuf::from("/r/a")));
        assert_eq!(actual.get(1), Some(&PathBuf::from("/r/a/b")));
    }

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

    #[test]
    fn strips_compiler_dependency_spellings() {
        for (verbatim, plain) in [
            ("\\\\?\\C:\\app\\src\\types.ts", "C:\\app\\src\\types.ts"),
            ("\\\\?\\c:\\app\\tsconfig.json", "c:\\app\\tsconfig.json"),
            ("\\\\?\\C:\\", "C:\\"),
            (
                "\\\\?\\C:\\.vize\\a.b\\console.ts",
                "C:\\.vize\\a.b\\console.ts",
            ),
            ("\\\\?\\unc\\host\\share", "\\\\host\\share"),
            ("//?/UNC/host/share/app", "//host/share/app"),
        ] {
            assert_eq!(
                strip_windows_verbatim_prefix(verbatim).as_deref(),
                Some(plain)
            );
        }
    }

    #[test]
    fn keeps_verbatim_paths_without_a_plain_spelling() {
        for verbatim in [
            "\\\\?\\C:",
            "\\\\?\\C:app",
            "\\\\?\\1:\\app",
            "\\\\?\\Volume{01234567-89ab-cdef-0123-456789abcdef}\\app",
            "\\\\?\\GLOBALROOT\\Device\\HarddiskVolume1\\app",
            "\\\\?\\UNC\\host",
            "\\\\?\\UNC\\\\share\\app",
            "\\\\?\\C:\\app\\..\\secret",
            "\\\\?\\C:\\app\\.\\file.ts",
            "\\\\?\\C:\\app\\trailing.",
            "\\\\?\\C:\\app\\trailing ",
            "\\\\?\\C:\\app\\con",
            "\\\\?\\C:\\app\\NUL.d.ts",
            "\\\\?\\C:\\app\\lpt¹.txt",
            "\\\\?\\C:\\app\\aux .ts",
            "\\\\?\\C:\\app\\a:b",
            "\\\\?\\C:\\app\\a/b",
            "\\\\?\\C:\\app\\a*b",
            "\\\\.\\C:\\app",
        ] {
            assert_eq!(strip_windows_verbatim_prefix(verbatim), None, "{verbatim}");
        }
        let long_name = ["\\\\?\\C:\\", &"a".repeat(256)].concat();
        assert_eq!(strip_windows_verbatim_prefix(&long_name), None);
    }

    #[test]
    fn strips_long_paths_whose_names_fit() {
        let deep = "node_modules\\".repeat(30);
        assert_eq!(
            strip_windows_verbatim_prefix(&["\\\\?\\C:\\", &deep].concat()).as_deref(),
            Some(["C:\\", &deep].concat().as_str())
        );
    }
}
