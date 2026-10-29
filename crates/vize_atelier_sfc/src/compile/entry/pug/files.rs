//! Filesystem includes preserve relative traversal and symlink behavior.
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use vize_l1_to_l2::lower::pug::{MAX_INCLUDE_BYTES, PugIncludes};

pub(super) struct FileIncludes;

impl PugIncludes for FileIncludes {
    fn read(
        &mut self,
        from: &str,
        path: &str,
    ) -> Result<(vize_l0::String, vize_l0::String), vize_l0::String> {
        let bytes = path.as_bytes();
        if path.starts_with(['/', '\\'])
            || (bytes.first().is_some_and(u8::is_ascii_alphabetic) && bytes.get(1) == Some(&b':'))
            || Path::new(path)
                .components()
                .any(|part| matches!(part, Component::Prefix(_) | Component::RootDir))
        {
            return Err(vize_l0::String::from(
                "absolute includes require a basedir option",
            ));
        }
        let base = Path::new(from).parent().unwrap_or(Path::new(""));
        let resolved = normalize(&base.join(path));
        let content =
            std::fs::File::open(&resolved).and_then(|file| read_bounded(file, MAX_INCLUDE_BYTES));
        match content {
            Ok(content) => Ok((
                vize_l0::String::from(resolved.to_string_lossy().as_ref()),
                content,
            )),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                Err(vize_l0::String::from("the file does not exist"))
            }
            Err(error) => Err(vize_l0::cstr!("{error}")),
        }
    }
}

fn read_bounded(reader: impl Read, limit: usize) -> io::Result<vize_l0::String> {
    let mut bytes = Vec::new();
    reader
        .take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pug include source exceeds 16 MiB",
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(vize_l0::String::from(text))
}

/// `path.join`'s lexical normalization of `.` and `..`.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(out.components().next_back(), Some(Component::Normal(_))) =>
            {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_forms_fail_before_filesystem_access_on_every_host() {
        for path in [
            "/pug/root.pug",
            r"\root.pug",
            r"C:\root.pug",
            "C:root.pug",
            r"\\server\share\root.pug",
            r"\\?\C:\root.pug",
        ] {
            let result = FileIncludes.read("/missing/main.pug", path);
            assert_eq!(
                result,
                Err(vize_l0::String::from(
                    "absolute includes require a basedir option"
                ))
            );
        }
    }

    #[test]
    fn oversized_reader_stops_after_the_single_probe_byte() {
        let mut reader = io::Cursor::new([b'x'; 128]);
        assert!(read_bounded(&mut reader, 8).is_err());
        assert_eq!(reader.position(), 9);
        assert!(
            matches!(read_bounded(io::Cursor::new(b"small"), 8), Ok(value) if value == "small")
        );
    }

    #[test]
    fn relative_parent_components_keep_their_existing_resolution() {
        assert_eq!(
            normalize(Path::new("root/nested/../sibling.pug")),
            Path::new("root/sibling.pug")
        );
        assert_eq!(
            normalize(Path::new("../shared.pug")),
            Path::new("../shared.pug")
        );
    }
}
