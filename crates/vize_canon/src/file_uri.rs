use std::path::{Path, PathBuf};

use vize_carton::String;

pub(crate) fn path_to_file_uri(path: &Path) -> String {
    let path = path.to_string_lossy();
    let mut uri = String::default();
    uri.push_str("file://");
    append_encoded_path(&mut uri, path.as_bytes());
    uri
}

pub(crate) fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let path = uri.strip_prefix("file://")?;
    let decoded = decode_path(path)?;
    Some(PathBuf::from(decoded.as_str()))
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
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    uri.push('%');
    uri.push(HEX[(byte >> 4) as usize] as char);
    uri.push(HEX[(byte & 0x0f) as usize] as char);
}

fn decode_path(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let mut decoded = std::vec::Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
        {
            decoded.push((high << 4) | low);
            index += 3;
            continue;
        }

        decoded.push(bytes[index]);
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
    use super::{file_uri_to_path, path_to_file_uri};
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
    fn decodes_windows_drive_letter_uris() {
        // The escaped drive colon must decode; the leading slash is kept
        // as-is (drive-letter normalization is out of scope here).
        assert_eq!(
            file_uri_to_path("file:///c%3A/work/App.vue"),
            Some(PathBuf::from("/c:/work/App.vue"))
        );
    }

    #[test]
    fn rejects_escape_sequences_that_are_not_valid_utf8() {
        assert_eq!(file_uri_to_path("file:///work/%FF%FE/App.vue"), None);
    }
}
