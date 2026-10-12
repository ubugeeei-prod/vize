// Based on https://github.com/getsentry/rust-sourcemap/blob/master/src/utils.rs
use std::borrow::Cow;
use std::iter::repeat;

pub fn is_abs_path(s: &str) -> bool {
    if s.starts_with('/') || s.starts_with('\\') {
        return true;
    } else if s.len() > 3 {
        let b = s.as_bytes();
        if b[1] == b':'
            && (b[2] == b'/' || b[2] == b'\\')
            && ((b[0] >= b'a' && b[0] <= b'z') || (b[0] >= b'A' && b[0] <= b'Z'))
        {
            return true;
        }
    }

    // fallback to false
    false
}

fn get_common_prefix_len<'a>(items: &'a [Cow<'a, [&'a str]>]) -> usize {
    if items.is_empty() {
        return 0;
    }

    let shortest = &items[0];
    let mut max_idx = None;
    for seq in items.iter() {
        let mut seq_max_idx = None;
        for (idx, &comp) in shortest.iter().enumerate() {
            if seq.get(idx) != Some(&comp) {
                break;
            }
            seq_max_idx = Some(idx);
        }
        if max_idx.is_none() || seq_max_idx < max_idx {
            max_idx = seq_max_idx;
        }
    }

    if let Some(max_idx) = max_idx {
        max_idx + 1
    } else {
        0
    }
}

fn chunk_path(p: &str) -> Vec<&str> {
    return p
        .split(&['/', '\\'][..])
        .filter(|x| !x.is_empty() && *x != ".")
        .collect();
}

// Helper function to calculate the path from a base file to a target file.
pub fn make_relative_path(base: &str, target: &str) -> String {
    let mut target_str = target;
    {
        let target_lower = target.to_ascii_lowercase();
        if target_lower.strip_prefix("file://").is_some() {
            target_str = &target[7..];
        }
    }

    if !is_abs_path(target_str) {
        if target_str.contains(':') {
            String::from(target_str)
        } else {
            return chunk_path(target_str).join("/");
        }
    } else {
        let target_path: Vec<&str> = chunk_path(target_str);
        let base_dir: Vec<&str> = chunk_path(base);
        let items = vec![
            Cow::Borrowed(base_dir.as_slice()),
            Cow::Borrowed(target_path.as_slice()),
        ];
        let prefix_len = get_common_prefix_len(&items);
        let mut rel_list: Vec<&str> = repeat("..").take(base_dir.len() - prefix_len).collect();
        rel_list.extend_from_slice(&target_path[prefix_len..]);
        rel_list.join("/")
    }
}

#[test]
fn test_make_relative_path() {
    assert_eq!(
        &make_relative_path("/foo/bar", "/foo/bar/baz.map"),
        "baz.map"
    );
    assert_eq!(
        &make_relative_path("/foo/bar/.", "/foo/bar/baz.map"),
        "baz.map"
    );
    assert_eq!(
        &make_relative_path("/foo/bar", "/foo/baz.map"),
        "../baz.map"
    );
    assert_eq!(&make_relative_path("/some/abs/path", "foo.js"), "foo.js");
    assert_eq!(
        &make_relative_path("C:\\blah\\sub", "C:\\blah\\foo.js"),
        "../foo.js"
    );
    assert_eq!(&make_relative_path("/", "./test.js"), "test.js");
}
