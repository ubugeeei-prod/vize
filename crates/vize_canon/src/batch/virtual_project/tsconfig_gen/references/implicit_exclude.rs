//! TypeScript rejects package folders only when a wildcard include segment
//! consumes them; literal `node_modules` segments remain valid program roots.

use std::{ffi::OsStr, path::Path};

use super::spec::names_equal;

const PACKAGE_FOLDERS: [&str; 3] = ["node_modules", "bower_components", "jspm_packages"];

pub(super) fn wildcard_hits_package_folder(
    pattern: &str,
    relative: &Path,
    case_sensitive: bool,
) -> bool {
    let path = relative
        .components()
        .map(|component| component.as_os_str())
        .collect::<Vec<_>>();
    let segments = pattern
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let mut head = 0;
    let mut tail = path.len();
    let Some(recursive_at) = segments.iter().position(|segment| *segment == "**") else {
        return segments.iter().enumerate().any(|(index, segment)| {
            segment.contains(['*', '?', '['])
                && path
                    .get(index)
                    .is_some_and(|name| is_package_folder(name, case_sensitive))
        });
    };

    for segment in &segments[..recursive_at] {
        if head >= tail {
            return false;
        }
        if segment.contains(['*', '?', '[']) && is_package_folder(path[head], case_sensitive) {
            return true;
        }
        head += 1;
    }
    for segment in segments[recursive_at + 1..].iter().rev() {
        if *segment == "**" || tail <= head {
            break;
        }
        tail -= 1;
        if segment.contains(['*', '?', '[']) && is_package_folder(path[tail], case_sensitive) {
            return true;
        }
    }
    path[head..tail]
        .iter()
        .any(|name| is_package_folder(name, case_sensitive))
}

fn is_package_folder(name: &OsStr, case_sensitive: bool) -> bool {
    PACKAGE_FOLDERS
        .iter()
        .any(|folder| names_equal(name, OsStr::new(folder), case_sensitive))
}
