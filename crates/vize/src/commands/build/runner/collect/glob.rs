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
        let normalized =
            normalize_separators(input.get(current_dir_prefix_len..).unwrap_or_default());
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

    pub(super) fn matches(&self, path: &Path) -> bool {
        let candidate = path.to_string_lossy();
        let prefix_len = current_dir_prefix_len(candidate.as_ref());
        let candidate = normalize_separators(candidate.get(prefix_len..).unwrap_or_default());
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
    // Build globs always contain a metacharacter; a literal path is its own
    // root.
    pattern.find(GLOB_METACHARACTERS).unwrap_or(pattern.len())
}

fn literal_root(pattern: &str) -> PathBuf {
    let metacharacter = first_metacharacter(pattern);
    let prefix = pattern.get(..metacharacter).unwrap_or_default();

    prefix.rfind(['/', '\\']).map_or_else(
        || PathBuf::from("."),
        |separator| {
            let root = normalize_separators(prefix.get(..=separator).unwrap_or_default());
            PathBuf::from(root.as_ref())
        },
    )
}

fn maximum_depth(pattern: &str) -> Option<usize> {
    let metacharacter = first_metacharacter(pattern);
    let suffix_start = pattern
        .get(..metacharacter)
        .unwrap_or_default()
        .rfind(std::path::MAIN_SEPARATOR)
        .map_or(0, |separator| separator + 1);
    let mut depth = 0;

    for component in pattern
        .get(suffix_start..)
        .unwrap_or_default()
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
mod tests {
    #![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]

    use super::BuildGlob;
    use std::path::{Path, PathBuf};

    #[test]
    fn matches_segment_wildcards_and_character_classes() {
        let single_segment = BuildGlob::new("src/*/App?.vue").unwrap();
        assert!(single_segment.matches(Path::new("src/admin/App1.vue")));
        assert!(!single_segment.matches(Path::new("src/admin/deep/App1.vue")));
        assert!(!single_segment.matches(Path::new("src/admin/App12.vue")));

        let class = BuildGlob::new("src/[AB]*[0-9].vue").unwrap();
        assert!(class.matches(Path::new("src/App2.vue")));
        assert!(class.matches(Path::new("src/Button7.vue")));
        assert!(!class.matches(Path::new("src/Card7.vue")));
    }

    #[test]
    fn recursive_wildcards_match_zero_or_many_directories() {
        let recursive = BuildGlob::new("src/**/App.vue").unwrap();
        assert_eq!(recursive.max_depth(), None);
        assert!(recursive.matches(Path::new("src/App.vue")));
        assert!(recursive.matches(Path::new("src/a/b/App.vue")));
        assert!(!recursive.matches(Path::new("other/src/App.vue")));
    }

    #[test]
    fn rejects_invalid_patterns_with_original_positions() {
        let unclosed = BuildGlob::new("./src/[AB.vue").unwrap_err();
        assert_eq!(
            unclosed.to_string(),
            "Pattern syntax error near position 6: invalid range pattern"
        );

        let embedded_recursive = BuildGlob::new("src/foo**/App.vue").unwrap_err();
        assert_eq!(
            embedded_recursive.to_string(),
            "Pattern syntax error near position 6: recursive wildcards must form a single path component"
        );
    }

    #[test]
    fn preserves_deep_literal_prefixes_and_absolute_roots() {
        let nested = BuildGlob::new("workspace/packages/ui/src/App?.vue").unwrap();
        assert_eq!(nested.root(), Path::new("workspace/packages/ui/src/"));

        let absolute = BuildGlob::new("/workspace/src/**/*.vue").unwrap();
        assert_eq!(absolute.root(), Path::new("/workspace/src/"));
        assert!(absolute.matches(Path::new("/workspace/src/nested/App.vue")));
        assert!(!absolute.matches(Path::new("/other/workspace/src/App.vue")));

        assert_eq!(BuildGlob::new("/**/*.vue").unwrap().root(), Path::new("/"));
        let windows_root = BuildGlob::new(r"C:\**\*.vue").unwrap();
        let expected_windows_root = if cfg!(windows) { r"C:\" } else { "C:/" };
        assert_eq!(windows_root.root().to_string_lossy(), expected_windows_root);
    }

    #[test]
    fn bracket_expressions_escape_metacharacters() {
        let escaped = BuildGlob::new("src/Component[[]old[]]-[*]-[?].vue").unwrap();
        assert!(escaped.matches(Path::new("src/Component[old]-*-?.vue")));
        assert!(!escaped.matches(Path::new("src/Componentxold]-*-?.vue")));
    }

    #[test]
    fn accepts_windows_separators_portably() {
        let pattern = BuildGlob::new(r"C:\workspace\src\**\Card?.vue").unwrap();
        assert!(pattern.matches(Path::new(r"C:\workspace\src\nested\Card1.vue")));
        assert!(!pattern.matches(Path::new(r"C:\workspace\other\nested\Card1.vue")));
    }

    #[test]
    fn uses_native_case_sensitivity() {
        let pattern = BuildGlob::new("src/card?.vue").unwrap();
        assert_eq!(pattern.matches(Path::new("src/CardA.vue")), cfg!(windows));
    }

    #[cfg(windows)]
    #[test]
    fn windows_matching_is_case_insensitive_and_accepts_forward_slashes() {
        let pattern = BuildGlob::new("C:/workspace/src/card?.vue").unwrap();
        assert!(pattern.matches(Path::new(r"C:\workspace\src\CardA.vue")));
    }

    #[test]
    fn glob_root_without_a_literal_directory_is_current_directory() {
        let root_pattern = BuildGlob::new("*.vue").unwrap();
        assert_eq!(root_pattern.root(), PathBuf::from("."));
        assert_eq!(root_pattern.max_depth(), Some(1));

        let nested = BuildGlob::new("src/*/App?.vue").unwrap();
        assert_eq!(nested.max_depth(), Some(2));
    }
}
