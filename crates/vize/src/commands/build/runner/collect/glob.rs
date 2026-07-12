//! Compiled build-input glob matching.
//!
//! Existing paths are classified before this module is used, so metacharacters
//! in an existing file or directory remain literal. For actual patterns,
//! backslashes are portable path separators rather than escapes. Literal glob
//! metacharacters use bracket expressions: `[*]`, `[?]`, `[[]`, and `[]]`.

use std::path::{Path, PathBuf};

use ::glob::{MatchOptions, Pattern, PatternError};

pub(super) const GLOB_METACHARACTERS: [char; 3] = ['*', '?', '['];

#[derive(Debug)]
pub(super) struct BuildGlob {
    root: PathBuf,
    pattern: Pattern,
    max_depth: Option<usize>,
}

impl BuildGlob {
    pub(super) fn new(input: &str) -> Result<Self, PatternError> {
        let current_dir_prefix_len = current_dir_prefix_len(input);
        let normalized = normalize_separators(&input[current_dir_prefix_len..]);
        let pattern = Pattern::new(normalized.as_ref()).map_err(|error| PatternError {
            pos: error.pos + current_dir_prefix_len,
            msg: error.msg,
        })?;

        Ok(Self {
            root: literal_root(input),
            pattern,
            max_depth: maximum_depth(normalized.as_ref()),
        })
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) const fn max_depth(&self) -> Option<usize> {
        self.max_depth
    }

    #[allow(clippy::disallowed_methods)]
    pub(super) fn matches(&self, path: &Path) -> bool {
        let candidate = path.to_string_lossy();
        let prefix_len = current_dir_prefix_len(candidate.as_ref());
        let candidate = normalize_separators(&candidate[prefix_len..]);
        self.pattern
            .matches_with(candidate.as_ref(), match_options())
    }
}

fn normalize_separators(value: &str) -> std::borrow::Cow<'_, str> {
    #[cfg(windows)]
    let (foreign, native) = ('/', "\\");
    #[cfg(not(windows))]
    let (foreign, native) = ('\\', "/");

    if value.contains(foreign) {
        value.replace(foreign, native).into()
    } else {
        value.into()
    }
}

fn current_dir_prefix_len(value: &str) -> usize {
    let mut prefix_len = 0;
    let mut rest = value;
    while let Some(stripped) = rest.strip_prefix("./").or_else(|| rest.strip_prefix(".\\")) {
        prefix_len += 2;
        rest = stripped;
    }
    prefix_len
}

fn first_metacharacter(pattern: &str) -> usize {
    pattern
        .find(GLOB_METACHARACTERS)
        .expect("build globs always contain a metacharacter")
}

fn literal_root(pattern: &str) -> PathBuf {
    let metacharacter = first_metacharacter(pattern);
    let prefix = &pattern[..metacharacter];

    prefix.rfind(['/', '\\']).map_or_else(
        || PathBuf::from("."),
        |separator| {
            let root = normalize_separators(&prefix[..=separator]);
            PathBuf::from(root.as_ref())
        },
    )
}

fn maximum_depth(pattern: &str) -> Option<usize> {
    let metacharacter = first_metacharacter(pattern);
    let suffix_start = pattern[..metacharacter]
        .rfind(std::path::MAIN_SEPARATOR)
        .map_or(0, |separator| separator + 1);
    let mut depth = 0;

    for component in pattern[suffix_start..]
        .split(std::path::MAIN_SEPARATOR)
        .filter(|component| !component.is_empty())
    {
        if component == "**" {
            return None;
        }
        depth += 1;
    }
    Some(depth)
}

const fn match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}

#[cfg(test)]
#[path = "glob_tests.rs"]
mod tests;
