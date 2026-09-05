//! File collection and glob pattern matching for the build command.

use std::{
    fmt,
    path::{Path, PathBuf},
};

use ignore::WalkBuilder;

use self::glob::{BuildGlob, GLOB_METACHARACTERS};

mod glob;

#[derive(Debug)]
pub(super) struct CollectedFiles {
    pub files: Vec<PathBuf>,
    pub roots: Vec<PathBuf>,
}

#[derive(Debug)]
pub(super) enum InputError<'a> {
    MissingLiteral {
        input: &'a str,
    },
    NonVueFile {
        input: &'a str,
    },
    InvalidGlob {
        input: &'a str,
        error: ::glob::PatternError,
    },
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
            Self::InvalidGlob { input, error } => {
                write!(formatter, "Invalid glob pattern {input}: {error}")
            }
        }
    }
}

#[derive(Debug)]
enum BuildInput<'a> {
    File(&'a Path),
    Directory(&'a Path),
    Glob(BuildGlob),
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
            BuildInput::Glob(pattern) => {
                let root = pattern.root();
                if root.is_dir() {
                    roots.push(root.to_path_buf());
                }
                collect_walked_files(root, Some(&pattern), &mut files);
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
        BuildGlob::new(input)
            .map(BuildInput::Glob)
            .map_err(|error| InputError::InvalidGlob { input, error })
    } else {
        Err(InputError::MissingLiteral { input })
    }
}

fn collect_walked_files(root: &Path, pattern: Option<&BuildGlob>, files: &mut Vec<PathBuf>) {
    let mut walker = WalkBuilder::new(root);
    if pattern.is_some() {
        walker.hidden(false);
    }
    if let Some(max_depth) = pattern.and_then(BuildGlob::max_depth) {
        walker.max_depth(Some(max_depth));
    }

    for entry in walker.build().flatten() {
        let path = entry.path();

        if path.is_file()
            && is_vue_file(path)
            && pattern.is_none_or(|pattern| pattern.matches(path))
        {
            files.push(path.to_path_buf());
        }
    }
}

#[inline]
fn contains_glob_metacharacter(input: &str) -> bool {
    input.contains(GLOB_METACHARACTERS)
}

#[inline]
fn is_vue_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}

#[cfg(test)]
mod tests;
