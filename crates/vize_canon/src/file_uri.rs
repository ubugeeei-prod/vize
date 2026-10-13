use std::path::{Path, PathBuf};

use vize_carton::String;

/// How the host spells absolute paths. Windows paths must become
/// `file:///C:/dir/file` URIs: a native checker parses `file://C:%5Cdir` as
/// host `C:` and cannot find the document (#8940).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathStyle {
    Posix,
    Windows,
}

const HOST_STYLE: PathStyle = if cfg!(windows) {
    PathStyle::Windows
} else {
    PathStyle::Posix
};

pub(crate) fn path_to_file_uri(path: &Path) -> String {
    path_to_file_uri_in(&path.to_string_lossy(), HOST_STYLE)
}

pub(crate) fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    file_uri_to_path_in(uri, HOST_STYLE).map(|path| PathBuf::from(path.as_str()))
}

fn path_to_file_uri_in(path: &str, style: PathStyle) -> String {
    let mut uri = String::default();
    uri.push_str("file://");
    if style == PathStyle::Posix {
        append_encoded_path(&mut uri, path.as_bytes());
        return uri;
    }
    let slashed = path.replace('\\', "/");
    // Verbatim prefixes (`\\?\C:\`, `\\?\UNC\server\share`) name the same file.
    let slashed = match slashed.strip_prefix("//?/") {
        Some(rest) => match rest.strip_prefix("UNC/") {
            Some(share) => String::from("//") + share,
            None => String::from(rest),
        },
        None => String::from(slashed.as_str()),
    };
    if let Some(share) = slashed.strip_prefix("//") {
        // A UNC share's server is the URI authority.
        append_encoded_path(&mut uri, share.as_bytes());
        return uri;
    }
    if !slashed.starts_with('/') {
        uri.push('/');
    }
    append_encoded_path(&mut uri, slashed.as_bytes());
    uri
}

fn file_uri_to_path_in(uri: &str, style: PathStyle) -> Option<String> {
    let path = uri.strip_prefix("file://")?;
    let decoded = decode_path(path)?;
    if style == PathStyle::Posix {
        return Some(decoded);
    }
    if let Some(local) = decoded.strip_prefix('/') {
        // `/C:/dir` and `/c%3A/dir` are drive paths; a bare `/dir` stays rooted.
        let local = if has_drive_prefix(local) {
            local
        } else {
            decoded.as_str()
        };
        return Some(String::from(local.replace('/', "\\").as_str()));
    }
    if decoded.is_empty() {
        return None;
    }
    // `file://server/share` is a UNC path.
    Some(String::from("\\\\") + decoded.replace('/', "\\").as_str())
}

fn has_drive_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    matches!(bytes, [letter, b':', ..] if letter.is_ascii_alphabetic())
        && matches!(bytes.get(2), None | Some(b'/'))
}

fn append_encoded_path(uri: &mut String, path: &[u8]) {
    for &byte in path {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                uri.push(byte as char)
            }
            _ => append_percent_encoded(uri, byte),
        }
    }
}

fn append_percent_encoded(uri: &mut String, byte: u8) {
    uri.push('%');
    for nibble in [byte >> 4, byte & 0x0f] {
        if let Some(digit) = char::from_digit(u32::from(nibble), 16) {
            uri.push(digit.to_ascii_uppercase());
        }
    }
}

fn decode_path(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut decoded = std::vec::Vec::with_capacity(bytes.len());
    let mut index = 0;

    while let Some(&byte) = bytes.get(index) {
        if let Some(&[b'%', high, low]) = bytes.get(index..index + 3)
            && let (Some(high), Some(low)) = (hex_value(high), hex_value(low))
        {
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }

        decoded.push(byte);
        index += 1;
    }

    let decoded = std::str::from_utf8(&decoded).ok()?;
    Some(String::from(decoded))
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PathStyle, file_uri_to_path, file_uri_to_path_in, path_to_file_uri, path_to_file_uri_in,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn encodes_reserved_file_uri_path_bytes() {
        assert_eq!(
            path_to_file_uri(Path::new("/workspace/pages/[[org]]/[name] #1.vue.ts")),
            "file:///workspace/pages/%5B%5Borg%5D%5D/%5Bname%5D%20%231.vue.ts"
        );
    }

    #[test]
    fn decodes_file_uri_path_bytes() {
        assert_eq!(
            file_uri_to_path("file:///workspace/pages/%5B%5Borg%5D%5D/%5Bname%5D%20%231.vue.ts"),
            Some(PathBuf::from("/workspace/pages/[[org]]/[name] #1.vue.ts"))
        );
    }

    #[test]
    fn decodes_multi_byte_utf8_escapes() {
        // Multi-byte sequences must be assembled from the decoded bytes;
        // pushing each byte as a `char` would produce mojibake.
        assert_eq!(
            file_uri_to_path("file:///Users/foo/%E3%83%86%E3%82%B9%E3%83%88/App.vue"),
            Some(PathBuf::from("/Users/foo/テスト/App.vue"))
        );
    }

    #[test]
    fn round_trips_non_ascii_paths() {
        let path = Path::new("/Users/foo/テスト/App.vue");
        assert_eq!(
            path_to_file_uri(path),
            "file:///Users/foo/%E3%83%86%E3%82%B9%E3%83%88/App.vue"
        );
        assert_eq!(
            file_uri_to_path(&path_to_file_uri(path)),
            Some(path.to_path_buf())
        );
    }

    #[test]
    fn keeps_drive_like_posix_segments_on_posix_hosts() {
        assert_eq!(
            file_uri_to_path_in("file:///c%3A/work/App.vue", PathStyle::Posix).as_deref(),
            Some("/c:/work/App.vue")
        );
    }

    #[test]
    fn spells_windows_drive_paths_as_rooted_file_uris() {
        // #8940: the fresh-project smoke runs under a path with a space and
        // non-ASCII characters. A native checker must read it as a drive path.
        let path = r"C:\Users\runner\fresh\vite vue 日本語\src\App.vue.ts";
        let uri = path_to_file_uri_in(path, PathStyle::Windows);
        assert_eq!(
            uri,
            "file:///C:/Users/runner/fresh/vite%20vue%20%E6%97%A5%E6%9C%AC%E8%AA%9E/src/App.vue.ts"
        );
        assert!(uri.parse::<lsp_types::Uri>().is_ok());
        assert_eq!(
            file_uri_to_path_in(&uri, PathStyle::Windows).as_deref(),
            Some(path)
        );
    }

    #[test]
    fn reads_client_spelled_windows_drive_uris() {
        for uri in ["file:///c%3A/work/App.vue", "file:///C:/work/App.vue"] {
            let path = file_uri_to_path_in(uri, PathStyle::Windows).unwrap();
            assert!(
                path.eq_ignore_ascii_case(r"c:\work\App.vue"),
                "{uri} -> {path}"
            );
        }
    }

    #[test]
    fn spells_windows_shares_and_verbatim_paths() {
        assert_eq!(
            path_to_file_uri_in(r"\\server\share\App.vue", PathStyle::Windows),
            "file://server/share/App.vue"
        );
        assert_eq!(
            file_uri_to_path_in("file://server/share/App.vue", PathStyle::Windows).as_deref(),
            Some(r"\\server\share\App.vue")
        );
        assert_eq!(
            path_to_file_uri_in(r"\\?\C:\work\App.vue", PathStyle::Windows),
            "file:///C:/work/App.vue"
        );
        assert_eq!(
            path_to_file_uri_in(r"\\?\UNC\server\share\App.vue", PathStyle::Windows),
            "file://server/share/App.vue"
        );
    }

    #[test]
    fn rejects_escape_sequences_that_are_not_valid_utf8() {
        assert_eq!(file_uri_to_path("file:///work/%FF%FE/App.vue"), None);
    }
}
