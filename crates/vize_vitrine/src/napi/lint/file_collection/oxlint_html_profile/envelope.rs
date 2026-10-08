use std::fs;
use std::path::{Component, Path, PathBuf};
use vize_l0::ToCompactString;

use super::{Refusal, RefusalKind, Request, Selection, SourceCustody, SourceRole};

mod entry;
pub(super) use entry::entry_refusal;

pub(super) struct Admitted {
    pub target: PathBuf,
    pub repository: PathBuf,
    pub is_dir: bool,
}

pub(super) fn admit(request: &Request<'_>) -> Result<Admitted, Refusal> {
    if !cfg!(unix) {
        return Err(Refusal::new(
            RefusalKind::NonPosix,
            request.cwd,
            "POSIX paths only",
        ));
    }
    if !request.cwd.is_absolute()
        || !request.cwd.is_dir()
        || request.cwd.to_str().is_none()
        || request
            .cwd
            .components()
            .any(|part| part == Component::ParentDir)
    {
        return Err(Refusal::new(
            RefusalKind::InvalidCwd,
            request.cwd,
            "absolute UTF-8 directory required",
        ));
    }
    let literal = request.literal_target;
    if literal.is_empty()
        || literal == "-"
        || literal.contains(['*', '?', '['])
        || Path::new(literal)
            .components()
            .any(|part| part == Component::ParentDir)
    {
        return Err(Refusal::new(
            RefusalKind::NonLiteralTarget,
            Path::new(literal),
            "one literal non-stdin target required",
        ));
    }
    let target = std::path::absolute(request.cwd.join(literal))
        .map_err(|error| Refusal::io(Path::new(literal), error))?;
    if !target.starts_with(request.cwd) {
        return Err(Refusal::new(
            RefusalKind::OutsideCwd,
            &target,
            "target must be within cwd",
        ));
    }
    let metadata = fs::symlink_metadata(&target).map_err(|error| {
        let kind = if error.kind() == std::io::ErrorKind::NotFound {
            RefusalKind::MissingTarget
        } else {
            RefusalKind::Io
        };
        Refusal::new(kind, &target, error.to_compact_string())
    })?;
    if !metadata.is_dir() && !metadata.is_file() && !metadata.is_symlink() {
        return Err(Refusal::new(
            RefusalKind::UnsupportedFileType,
            &target,
            "regular file or directory required",
        ));
    }
    let start = if metadata.is_dir() {
        target.as_path()
    } else {
        target.parent().unwrap_or(&target)
    };
    let mut repository = None;
    for directory in start.ancestors() {
        match fs::symlink_metadata(directory.join(".jj")) {
            Ok(_) => {
                return Err(Refusal::new(
                    RefusalKind::JujutsuMetadata,
                    directory,
                    "Jujutsu is outside the finite profile",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(Refusal::io(directory, error)),
        }
        let marker = directory.join(".git");
        match fs::symlink_metadata(&marker) {
            Ok(metadata) if metadata.is_dir() => {
                repository = Some(directory.to_path_buf());
                break;
            }
            Ok(_) => {
                return Err(Refusal::new(
                    RefusalKind::LinkedGitMetadata,
                    &marker,
                    "regular .git directory required",
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(Refusal::io(&marker, error)),
        }
    }
    let Some(repository) = repository else {
        return Err(Refusal::new(
            RefusalKind::MissingGitBoundary,
            &target,
            "regular .git ancestor required",
        ));
    };
    let suppression = request.cwd.join("oxlint-suppressions.json");
    match fs::symlink_metadata(&suppression) {
        Ok(_) => {
            return Err(Refusal::new(
                RefusalKind::SuppressionState,
                &suppression,
                "suppression state is unqualified",
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(Refusal::io(&suppression, error)),
    }
    if !request.cwd.starts_with(&repository) {
        return Err(Refusal::new(
            RefusalKind::NestedVcs,
            &repository,
            "cwd and target must share one boundary",
        ));
    }
    for path in target
        .ancestors()
        .take_while(|path| path.starts_with(&repository))
    {
        ensure_regular_path(path)?;
    }
    let custom = Path::new(request.custom_ignore_filename);
    if custom.components().count() != 1
        || !matches!(custom.components().next(), Some(Component::Normal(_)))
    {
        return Err(Refusal::new(
            RefusalKind::CustomIgnoreFilename,
            custom,
            "one custom-ignore filename required",
        ));
    }
    // Custom parents do not stop at Git boundaries. Refuse undeclared external
    // sources rather than silently substituting repository-only matching.
    if !request.no_ignore {
        for directory in repository.parent().into_iter().flat_map(Path::ancestors) {
            let path = directory.join(custom);
            match fs::symlink_metadata(&path) {
                Ok(_) => {
                    return Err(Refusal::new(
                        RefusalKind::ExternalCustomIgnore,
                        &path,
                        "custom ignore above owned boundary",
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(Refusal::io(&path, error)),
            }
        }
    }
    Ok(Admitted {
        target,
        repository,
        is_dir: metadata.is_dir(),
    })
}

fn ensure_regular_path(path: &Path) -> Result<(), Refusal> {
    let metadata = fs::symlink_metadata(path).map_err(|error| Refusal::io(path, error))?;
    if metadata.is_symlink() {
        return Err(Refusal::new(
            RefusalKind::Symlink,
            path,
            "symlink paths are unqualified",
        ));
    }
    Ok(())
}

pub(super) fn record_file(
    selection: &mut Selection,
    path: &Path,
    role: SourceRole,
) -> Result<(), Refusal> {
    if selection
        .sources
        .iter()
        .any(|source| source.path == path && source.role == role)
    {
        return Ok(());
    }
    let bytes = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_symlink() => {
            return Err(Refusal::new(
                RefusalKind::Symlink,
                path,
                "ignore/config symlink is unqualified",
            ));
        }
        Ok(metadata) if !metadata.is_file() => {
            return Err(Refusal::new(
                RefusalKind::UnsupportedFileType,
                path,
                "ignore/config source must be regular",
            ));
        }
        Ok(_) => Some(fs::read(path).map_err(|error| Refusal::io(path, error))?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(Refusal::io(path, error)),
    };
    selection.sources.push(SourceCustody {
        path: path.to_path_buf(),
        role,
        bytes,
    });
    Ok(())
}

pub(super) fn record_directory(
    request: &Request<'_>,
    directory: &Path,
    selection: &mut Selection,
) -> Result<(), Refusal> {
    record_file(
        selection,
        &directory.join(".gitignore"),
        SourceRole::GitIgnore,
    )?;
    record_file(
        selection,
        &directory.join(request.custom_ignore_filename),
        SourceRole::CustomIgnore,
    )
}

pub(super) fn record_ancestors(
    request: &Request<'_>,
    target: &Path,
    selection: &mut Selection,
) -> Result<(), Refusal> {
    let directory = if target.is_dir() {
        target
    } else {
        target.parent().unwrap_or(target)
    };
    let repository = selection.repository.clone();
    for parent in directory
        .ancestors()
        .take_while(|path| path.starts_with(&repository))
    {
        record_directory(request, parent, selection)?;
    }
    let info = repository.join(".git/info");
    match fs::symlink_metadata(&info) {
        Ok(metadata) if metadata.is_symlink() => {
            return Err(Refusal::new(
                RefusalKind::Symlink,
                &info,
                "Git info symlink is unqualified",
            ));
        }
        Ok(metadata) if !metadata.is_dir() => {
            return Err(Refusal::new(
                RefusalKind::UnsupportedFileType,
                &info,
                "Git info directory required",
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(Refusal::io(&info, error)),
    }
    record_file(selection, &info.join("exclude"), SourceRole::GitInfoExclude)
}
