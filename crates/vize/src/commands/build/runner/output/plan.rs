use std::path::{Component, Path, PathBuf};

use vize_s0::{FxHashSet, String};

use super::{OutputError, OutputFormat, ScriptExtension};

pub(crate) struct PlannedInput {
    pub(crate) source: PathBuf,
    pub(super) relative_source: PathBuf,
}

struct ComparableSource {
    source: PathBuf,
    normalized: PathBuf,
    layout_key: String,
}

pub(crate) fn plan_inputs(
    files: Vec<PathBuf>,
    roots: &[PathBuf],
) -> Result<Vec<PlannedInput>, OutputError> {
    let cwd = std::env::current_dir().map_err(OutputError::CurrentDirectory)?;
    let mut normalized_roots: Vec<_> = roots
        .iter()
        .map(|root| normalize_absolute(root, &cwd))
        .collect();
    normalized_roots.sort();
    normalized_roots.dedup();

    let normalized_files: Vec<_> = files
        .into_iter()
        .map(|source| {
            let normalized = normalize_absolute(&source, &cwd);
            (source, normalized)
        })
        .collect();
    // Lexical normalization defines output layout, not filesystem identity.
    // Resolve identity only for paths competing for the same portable layout
    // so the ordinary one-file-per-layout path stays syscall-free.
    let normalized_files = deduplicate_source_aliases(normalized_files, &cwd);

    if normalized_roots.is_empty() {
        normalized_roots.extend(
            normalized_files
                .iter()
                .filter_map(|(_, source)| source.parent().map(Path::to_path_buf)),
        );
        normalized_roots.sort();
        normalized_roots.dedup();
    }

    let root = common_root(&normalized_roots).ok_or(OutputError::NoCommonRoot)?;
    normalized_files
        .into_iter()
        .map(|(source, normalized)| {
            let relative_source =
                normalized
                    .strip_prefix(&root)
                    .map_err(|_| OutputError::SourceOutsideRoot {
                        source: source.clone(),
                        root: root.clone(),
                    })?;
            Ok(PlannedInput {
                source,
                relative_source: relative_source.to_path_buf(),
            })
        })
        .collect()
}

fn deduplicate_source_aliases(
    normalized_files: Vec<(PathBuf, PathBuf)>,
    cwd: &Path,
) -> Vec<(PathBuf, PathBuf)> {
    let mut keyed: Vec<_> = normalized_files
        .into_iter()
        .map(|(source, normalized)| ComparableSource {
            layout_key: portable_key(&normalized),
            source,
            normalized,
        })
        .collect();
    keyed.sort_by(|left, right| {
        left.layout_key
            .cmp(&right.layout_key)
            .then_with(|| left.normalized.cmp(&right.normalized))
            .then_with(|| left.source.cmp(&right.source))
    });
    let mut files = keyed.into_iter().peekable();
    let mut unique = Vec::with_capacity(files.len());
    let mut identities = FxHashSet::default();

    while let Some(first) = files.next() {
        let has_same_layout = files
            .peek()
            .is_some_and(|candidate| candidate.layout_key == first.layout_key);
        if !has_same_layout {
            unique.push((first.source, first.normalized));
            continue;
        }

        identities.clear();
        identities.insert(source_identity(&first.source, cwd));
        let layout_key = first.layout_key.clone();
        unique.push((first.source, first.normalized));
        while files
            .peek()
            .is_some_and(|candidate| candidate.layout_key == layout_key)
        {
            let candidate = files.next().expect("peeked source must exist");
            if identities.insert(source_identity(&candidate.source, cwd)) {
                unique.push((candidate.source, candidate.normalized));
            }
        }
    }
    unique.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));
    unique
}

fn source_identity(source: &Path, cwd: &Path) -> PathBuf {
    let absolute = if source.is_absolute() {
        source.to_path_buf()
    } else {
        cwd.join(source)
    };
    // If identity cannot be proven, keep the spelling distinct and let the
    // collision preflight fail closed instead of silently dropping a source.
    std::fs::canonicalize(&absolute).unwrap_or(absolute)
}

pub(crate) fn preflight_outputs(
    inputs: &[PlannedInput],
    output_directory: &Path,
    format: OutputFormat,
    script_extension: ScriptExtension,
) -> Result<(), OutputError> {
    let Some(extension) = fixed_extension(format, script_extension) else {
        return Ok(());
    };
    let paths: Vec<_> = inputs
        .iter()
        .map(|input| OutputPath {
            source: &input.source,
            output: destination(input, output_directory, extension),
        })
        .collect();
    validate_output_paths(&paths, output_directory)
}

fn fixed_extension(
    format: OutputFormat,
    script_extension: ScriptExtension,
) -> Option<&'static str> {
    match (format, script_extension) {
        (OutputFormat::Js, ScriptExtension::Downcompile) => Some("js"),
        (OutputFormat::Json, _) => Some("json"),
        (OutputFormat::Js, ScriptExtension::Preserve) | (OutputFormat::Stats, _) => None,
    }
}

pub(super) fn destination(
    input: &PlannedInput,
    output_directory: &Path,
    extension: &str,
) -> PathBuf {
    output_directory.join(input.relative_source.with_extension(extension))
}

pub(super) struct OutputPath<'a> {
    pub source: &'a Path,
    pub output: PathBuf,
}

struct ComparableOutput<'a> {
    source: &'a Path,
    output: &'a Path,
    normalized: PathBuf,
    key: String,
}

pub(super) fn validate_output_paths(
    paths: &[OutputPath<'_>],
    output_directory: &Path,
) -> Result<(), OutputError> {
    let cwd = std::env::current_dir().map_err(OutputError::CurrentDirectory)?;
    let root = normalize_absolute(output_directory, &cwd);
    let mut comparable: Vec<_> = paths
        .iter()
        .map(|path| {
            let normalized = normalize_absolute(&path.output, &cwd);
            let key = portable_key(&normalized);
            ComparableOutput {
                source: path.source,
                output: &path.output,
                normalized,
                key,
            }
        })
        .collect();
    comparable.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.source.cmp(right.source))
    });

    for pair in comparable.windows(2) {
        if pair[0].key == pair[1].key {
            return Err(collision(&pair[0], &pair[1]));
        }
    }
    for path in &comparable {
        let mut parent = path.normalized.parent();
        while let Some(candidate) = parent {
            if candidate == root || !candidate.starts_with(&root) {
                break;
            }
            let key = portable_key(candidate);
            if let Ok(index) = comparable.binary_search_by(|entry| entry.key.cmp(&key)) {
                return Err(collision(&comparable[index], path));
            }
            parent = candidate.parent();
        }
    }
    Ok(())
}

fn collision(first: &ComparableOutput<'_>, second: &ComparableOutput<'_>) -> OutputError {
    OutputError::Collision {
        first_source: first.source.to_path_buf(),
        first_output: first.output.to_path_buf(),
        second_source: second.source.to_path_buf(),
        second_output: second.output.to_path_buf(),
    }
}

fn common_root(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut root = paths.first()?.clone();
    while !paths.iter().all(|path| path.starts_with(&root)) {
        if !root.pop() {
            return None;
        }
    }
    Some(root)
}

fn normalize_absolute(path: &Path, cwd: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    normalized
}

fn portable_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_lowercase()
        .into()
}
