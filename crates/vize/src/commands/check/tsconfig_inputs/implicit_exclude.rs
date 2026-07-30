//! TypeScript's implicit exclusion of package folders from wildcard `include`
//! segments.
//!
//! `tsc` does not implement the `node_modules` / `bower_components` /
//! `jspm_packages` default as an `exclude` entry. It bakes the exclusion into
//! how an `include` pattern is compiled: every segment that carries a wildcard —
//! and every directory a `**` consumes — is prefixed with
//! `(?!(node_modules|bower_components|jspm_packages)(/|$))`
//! (`implicitExcludePathRegexPattern`), while a literal segment is matched
//! literally. Three consequences, each verified against
//! `tsgo -p tsconfig.json --listFiles`:
//!
//! * The exclusion applies at *every* depth, not only directly below the
//!   tsconfig directory. `include: ["packages/**/*.ts"]` never collects
//!   `packages/a/node_modules/dep/index.ts`.
//! * It is not the `exclude` field, so an explicit `exclude: []` does not switch
//!   it off.
//! * Spelling a package folder out as a literal segment keeps it:
//!   `include: ["packages/*/node_modules/dep/*.ts"]` does collect that file.
//!
//! Anchoring the three directory names at the tsconfig directory only — the
//! shape [`super::glob::default_exclude_specs`] produces — therefore misses
//! nested copies, which is how files from a real nested `node_modules` became
//! program roots (#3385).

use std::path::Path;

/// `commonPackageFolders` in TypeScript's `utilities.ts`.
const COMMON_PACKAGE_FOLDERS: [&str; 3] = ["node_modules", "bower_components", "jspm_packages"];

const RECURSIVE_SEGMENT: &str = "**";

/// Whether `relative` — a path relative to the include spec's base directory —
/// has a package folder in a segment that `pattern` matched with a wildcard.
///
/// The pattern is aligned against the path segment by segment: literally from
/// the left up to the first `**`, literally from the right back to the last
/// `**`, and everything still unclaimed in the middle is what the `**` consumed.
/// A pattern without `**` therefore claims every segment one-to-one, and a
/// pattern with several `**` treats the whole span between the outermost two as
/// consumed — conservative in the same direction as `tsc`, whose `**` fragment
/// rejects a package folder at any of the depths it spans.
pub(super) fn wildcard_segment_hits_package_folder(pattern: &str, relative: &Path) -> bool {
    let segments = relative
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    let pattern_segments = pattern
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    // Unclaimed path segments live in `segments[head..tail]`.
    let mut head = 0usize;
    let mut tail = segments.len();

    let mut recursive_at = None;
    for (index, segment) in pattern_segments.iter().enumerate() {
        if *segment == RECURSIVE_SEGMENT {
            recursive_at = Some(index);
            break;
        }
        // Fewer path segments than leading pattern segments means the pattern
        // did not match this path at all; nothing is wildcard-claimed.
        if head >= tail {
            return false;
        }
        if is_wildcard_segment(segment) && is_package_folder(segments[head]) {
            return true;
        }
        head += 1;
    }

    let Some(recursive_at) = recursive_at else {
        return false;
    };

    for segment in pattern_segments[recursive_at + 1..].iter().rev() {
        if *segment == RECURSIVE_SEGMENT {
            break;
        }
        if tail <= head {
            return false;
        }
        if is_wildcard_segment(segment) && is_package_folder(segments[tail - 1]) {
            return true;
        }
        tail -= 1;
    }

    segments[head..tail].iter().copied().any(is_package_folder)
}

fn is_wildcard_segment(segment: &str) -> bool {
    segment.contains(['*', '?', '['])
}

fn is_package_folder(segment: &str) -> bool {
    COMMON_PACKAGE_FOLDERS.contains(&segment)
}
