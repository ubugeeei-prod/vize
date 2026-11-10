//! Plain spellings of Windows verbatim paths handed to host file watchers.

use std::path::PathBuf;
use vize_l0::String;

/// `\\?\C:\x` → `C:\x` and `\\?\UNC\host\share\x` → `\\host\share\x` (also the
/// `//?/` spelling tools print) on Windows, only when the plain spelling names
/// the same file, as `dunce::simplified` decides. `None` leaves `path` as is.
pub fn plain_windows_path(path: &str) -> Option<String> {
    if cfg!(windows) {
        strip_verbatim_prefix(path)
    } else {
        None
    }
}

/// [`plain_windows_path`] for an owned path.
pub fn plain_windows_path_buf(path: PathBuf) -> PathBuf {
    path.to_str()
        .and_then(plain_windows_path)
        .map_or(path, |plain| PathBuf::from(plain.as_str()))
}

// Length is not limited: Rust std and Node re-add the prefix for long paths.
fn strip_verbatim_prefix(value: &str) -> Option<String> {
    let (rest, separator) = value
        .strip_prefix("\\\\?\\")
        .map(|rest| (rest, '\\'))
        .or_else(|| value.strip_prefix("//?/").map(|rest| (rest, '/')))?;
    let mut components = rest.split(separator);
    let head = components.next()?;
    let mut plain = String::with_capacity(rest.len() + 2);
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
    use super::strip_verbatim_prefix;

    #[test]
    fn strips_compiler_dependency_spellings() {
        let deep = "node_modules\\".repeat(30);
        let long_verbatim = ["\\\\?\\C:\\", &deep].concat();
        let long_plain = ["C:\\", &deep].concat();
        for (verbatim, plain) in [
            ("\\\\?\\C:\\app\\src\\types.ts", "C:\\app\\src\\types.ts"),
            ("\\\\?\\c:\\app\\tsconfig.json", "c:\\app\\tsconfig.json"),
            ("\\\\?\\C:\\", "C:\\"),
            (
                "\\\\?\\C:\\.vize\\a.b\\console.ts",
                "C:\\.vize\\a.b\\console.ts",
            ),
            ("\\\\?\\UNC\\host\\share\\app", "\\\\host\\share\\app"),
            ("\\\\?\\unc\\host\\share", "\\\\host\\share"),
            ("//?/D:/repo/tsconfig.json", "D:/repo/tsconfig.json"),
            ("//?/UNC/host/share/app", "//host/share/app"),
            (long_verbatim.as_str(), long_plain.as_str()),
        ] {
            assert_eq!(
                strip_verbatim_prefix(verbatim).as_deref(),
                Some(plain),
                "{verbatim}"
            );
        }
    }

    #[test]
    fn keeps_paths_without_a_plain_spelling() {
        let long_name = ["\\\\?\\C:\\", &"a".repeat(256)].concat();
        for path in [
            "C:\\app\\types.ts",
            "\\\\host\\share\\types.ts",
            "/app/src/types.ts",
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
            long_name.as_str(),
        ] {
            assert_eq!(strip_verbatim_prefix(path), None, "{path}");
        }
    }
}
