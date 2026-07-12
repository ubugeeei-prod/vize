//! File collection and glob pattern matching for the build command.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use ignore::Walk;

#[derive(Debug)]
pub(super) struct CollectedFiles {
    pub files: Vec<PathBuf>,
    pub roots: Vec<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum InputError<'a> {
    MissingLiteral { input: &'a str },
    NonVueFile { input: &'a str },
}

impl fmt::Display for InputError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLiteral { input } => {
                write!(formatter, "Build input does not exist: {input}")
            }
            Self::NonVueFile { input } => {
                write!(formatter, "Build input is not a .vue file: {input}")
            }
        }
    }
}

#[derive(Debug)]
enum BuildInput<'a> {
    File(&'a Path),
    Directory(&'a Path),
    Glob { root: PathBuf, pattern: &'a str },
}

#[allow(clippy::disallowed_types)]
pub(super) fn collect_files_or_exit(patterns: &[std::string::String]) -> CollectedFiles {
    collect_files(patterns).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    })
}

/// Collect `.vue` files from literal files, directories, and glob patterns.
#[allow(clippy::disallowed_types)]
pub(super) fn collect_files<'a>(
    patterns: &'a [std::string::String],
) -> Result<CollectedFiles, InputError<'a>> {
    // Validate every literal before starting an expensive walk. A typo or an
    // unsupported file must not affect collection roots or scan other inputs.
    let inputs = patterns
        .iter()
        .map(|pattern| classify_input(pattern))
        .collect::<Result<Vec<_>, _>>()?;
    let mut files = Vec::new();
    let mut roots = Vec::new();

    for input in inputs {
        match input {
            BuildInput::File(path) => {
                files.push(path.to_path_buf());
                if let Some(parent) = path.parent() {
                    roots.push(parent.to_path_buf());
                }
            }
            BuildInput::Directory(root) => {
                roots.push(root.to_path_buf());
                collect_walked_files(root, None, &mut files);
            }
            BuildInput::Glob { root, pattern } => {
                if root.is_dir() {
                    roots.push(root.clone());
                }
                collect_walked_files(&root, Some(pattern), &mut files);
            }
        }
    }

    files.sort();
    files.dedup();
    roots.sort();
    roots.dedup();
    Ok(CollectedFiles { files, roots })
}

fn classify_input(input: &str) -> Result<BuildInput<'_>, InputError<'_>> {
    let path = Path::new(input);
    if let Ok(metadata) = path.metadata() {
        if metadata.is_file() {
            return if is_vue_file(path) {
                Ok(BuildInput::File(path))
            } else {
                Err(InputError::NonVueFile { input })
            };
        }
        return if metadata.is_dir() {
            Ok(BuildInput::Directory(path))
        } else {
            Err(InputError::MissingLiteral { input })
        };
    }

    if contains_glob_metacharacter(input) {
        Ok(BuildInput::Glob {
            root: glob_root(input),
            pattern: input,
        })
    } else {
        Err(InputError::MissingLiteral { input })
    }
}

fn glob_root(pattern: &str) -> PathBuf {
    let metacharacter = pattern
        .find(['*', '?', '['])
        .expect("glob roots are only derived from classified glob inputs");
    let literal_prefix = &pattern[..metacharacter];

    literal_prefix.rfind(['/', '\\']).map_or_else(
        || PathBuf::from("."),
        |separator| {
            // Retaining the separator preserves filesystem roots such as `/`
            // and `C:\\` while remaining equivalent for ordinary directories.
            PathBuf::from(&literal_prefix[..=separator])
        },
    )
}

fn collect_walked_files(root: &Path, pattern: Option<&str>, files: &mut Vec<PathBuf>) {
    for entry in Walk::new(root).flatten() {
        let path = entry.path();

        if path.is_file()
            && is_vue_file(path)
            && pattern.is_none_or(|pattern| pattern_matches(path, pattern))
        {
            files.push(path.to_path_buf());
        }
    }
}

#[inline]
fn contains_glob_metacharacter(input: &str) -> bool {
    input.contains(['*', '?', '['])
}

#[inline]
fn is_vue_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

/// Check whether a file path matches a glob-like pattern.
#[allow(clippy::disallowed_types, clippy::disallowed_methods)]
fn pattern_matches(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy().replace('\\', "/");
    let pattern = pattern.replace('\\', "/");

    if pattern == "./**/*.vue" || pattern == "**/*.vue" {
        return path_str.ends_with(".vue");
    }

    if pattern.contains("**/*.vue")
        && let Some(prefix_end) = pattern.find("**")
    {
        let prefix = &pattern[..prefix_end];
        let prefix_normalized = prefix.trim_end_matches('/');
        let has_prefix_dir = prefix_normalized.is_empty()
            || path_str.match_indices(prefix_normalized).any(|(idx, _)| {
                path_str.as_bytes().get(idx + prefix_normalized.len()) == Some(&b'/')
            });
        return has_prefix_dir && path_str.ends_with(".vue");
    }

    if pattern.ends_with(".vue") {
        if path_str == pattern {
            return true;
        }
        if !path_str.ends_with(pattern.as_str()) {
            return false;
        }

        let prefix_len = path_str.len() - pattern.len();
        let Some(separator_idx) = prefix_len.checked_sub(1) else {
            return false;
        };
        return path_str.as_bytes().get(separator_idx) == Some(&b'/');
    }

    path_str.ends_with(".vue")
}

#[cfg(test)]
mod tests;
