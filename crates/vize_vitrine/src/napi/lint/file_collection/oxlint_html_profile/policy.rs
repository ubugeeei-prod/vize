use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use ignore::WalkBuilder;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::overrides::{Override, OverrideBuilder};
use vize_l0::{String, ToCompactString, cstr};

use super::{Origin, Original, Refusal, RefusalKind, Request, Selection, SourceRole, envelope};

pub(super) fn root_config<T>(
    request: &Request<'_>,
    selection: &mut Selection,
    decode: impl FnOnce(&Path, &[u8]) -> Result<(serde_json::Value, T), Refusal>,
) -> Result<(Gitignore, T), Refusal> {
    let path = request.root_json;
    if !path.is_absolute()
        || path.parent() != Some(request.cwd)
        || path.extension().is_none_or(|extension| extension != "json")
    {
        return Err(Refusal::new(
            RefusalKind::RootJsonLocation,
            path,
            "explicit regular .json file directly in cwd required",
        ));
    }
    envelope::record_file(selection, path, SourceRole::RootJson)?;
    let bytes = selection
        .sources
        .iter()
        .find(|source| source.path == path)
        .and_then(|source| source.bytes.as_ref())
        .ok_or_else(|| {
            Refusal::new(
                RefusalKind::RootJsonLocation,
                path,
                "root JSON does not exist",
            )
        })?;
    let invalid = |kind, details: String| Refusal {
        kind,
        path: path.to_path_buf(),
        details,
        original_bytes: Some(bytes.clone()),
    };
    let (value, plan) = decode(path, bytes)?;
    let Some(object) = value.as_object() else {
        return Err(invalid(
            RefusalKind::RootJsonSyntax,
            "JSON object required".into(),
        ));
    };
    if object.contains_key("extends") {
        return Err(invalid(
            RefusalKind::ConfigInheritance,
            "inherited configuration is unqualified".into(),
        ));
    }
    if object
        .get("overrides")
        .is_some_and(|value| value.as_array().is_none_or(|array| !array.is_empty()))
    {
        return Err(invalid(
            RefusalKind::ConfigOverrides,
            "configuration overrides are unqualified".into(),
        ));
    }
    let mut builder = GitignoreBuilder::new(request.cwd);
    if let Some(patterns) = object.get("ignorePatterns") {
        let Some(patterns) = patterns.as_array() else {
            return Err(invalid(
                RefusalKind::RootJsonSyntax,
                "ignorePatterns must be a string array".into(),
            ));
        };
        for pattern in patterns {
            let Some(pattern) = pattern.as_str() else {
                return Err(invalid(
                    RefusalKind::RootJsonSyntax,
                    "ignorePatterns must be a string array".into(),
                ));
            };
            builder
                .add_line(Some(path.to_path_buf()), pattern)
                .map_err(|error| invalid(RefusalKind::IgnoreSyntax, error.to_compact_string()))?;
        }
    }
    builder
        .build()
        .map(|matcher| (matcher, plan))
        .map_err(|error| invalid(RefusalKind::IgnoreSyntax, error.to_compact_string()))
}

pub(super) fn cli_overrides(request: &Request<'_>) -> Result<Option<Override>, Refusal> {
    if request.no_ignore {
        return Ok(None);
    }
    let mut builder = OverrideBuilder::new(request.cwd);
    for pattern in request.cli_ignore_patterns {
        // This mechanical prefix is the pinned provider contract, including !!.
        let original = cstr!("!{pattern}");
        builder.add(&original).map_err(|error| Refusal {
            kind: RefusalKind::IgnoreSyntax,
            path: request.cwd.to_path_buf(),
            details: error.to_compact_string(),
            original_bytes: Some(pattern.as_bytes().to_vec()),
        })?;
    }
    builder.build().map(Some).map_err(|error| {
        Refusal::new(
            RefusalKind::IgnoreSyntax,
            request.cwd,
            error.to_compact_string(),
        )
    })
}

pub(super) fn explicit_custom_excluded(
    request: &Request<'_>,
    target: &Path,
    overrides: &Option<Override>,
) -> Result<bool, Refusal> {
    if request.no_ignore {
        return Ok(false);
    }
    let path = request.cwd.join(request.custom_ignore_filename);
    let (custom, error) = Gitignore::new(&path);
    if let Some(error) = error.filter(|_| path.exists()) {
        return Err(Refusal {
            kind: RefusalKind::IgnoreSyntax,
            path: path.clone(),
            details: error.to_compact_string(),
            original_bytes: fs::read(&path).ok(),
        });
    }
    // Deliberately direct matching, not ancestor matching; keep the independent
    // OR used for explicit files before traversal (custom whitelist cannot undo CLI).
    Ok(overrides
        .as_ref()
        .is_some_and(|matcher| matcher.matched(target, false).is_ignore())
        || custom.matched(target, false).is_ignore())
}

fn configure(builder: &mut WalkBuilder) -> &mut WalkBuilder {
    builder
        .hidden(false)
        .ignore(false)
        .git_global(false)
        .git_ignore(true)
        .parents(true)
        .git_exclude(true)
        .require_git(true)
}

pub(super) fn vcs_root_excluded(selection: &Selection) -> Result<bool, Refusal> {
    let mut builder = WalkBuilder::new(&selection.repository);
    configure(&mut builder);
    let Some(mut matcher) = builder.build_matchers().pop() else {
        return Err(Refusal::new(
            RefusalKind::Internal,
            &selection.repository,
            "one root matcher required",
        ));
    };
    let relative = selection
        .target
        .strip_prefix(&selection.repository)
        .map_err(|error| {
            Refusal::new(
                RefusalKind::Internal,
                &selection.target,
                error.to_compact_string(),
            )
        })?;
    let is_dir = fs::symlink_metadata(&selection.target)
        .map_err(|error| Refusal::io(&selection.target, error))?
        .is_dir();
    let (matched, error) = matcher.matched_with_errors(relative, is_dir);
    if let Some(error) = error {
        return Err(Refusal::new(
            RefusalKind::IgnoreSyntax,
            &selection.target,
            error.to_compact_string(),
        ));
    }
    Ok(matched.is_ignore())
}

#[expect(
    clippy::disallowed_types,
    reason = "ignore requires an owned Send+Sync 'static entry predicate and a shared refusal slot"
)]
pub(super) fn walk_originals(
    request: &Request<'_>,
    config: &Gitignore,
    overrides: Option<Override>,
    is_dir: bool,
    selection: &mut Selection,
) -> Result<(), Refusal> {
    // WalkBuilder's filter_entry bypasses depth zero. Oxlint's visitor does not.
    if is_dir
        && selection
            .target
            .file_name()
            .is_some_and(|name| name == ".git" || name == ".jj")
    {
        return Ok(());
    }
    let mut builder = WalkBuilder::new(&selection.target);
    if !request.no_ignore {
        builder.add_custom_ignore_filename(request.custom_ignore_filename);
        if let Some(overrides) = overrides {
            builder.overrides(overrides);
        }
    }
    configure(&mut builder).follow_links(true);
    let refusal = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&refusal);
    let repository = selection.repository.clone();
    let root_json = selection.root_json.clone();
    let custom = request.custom_ignore_filename.to_compact_string();
    builder.filter_entry(move |entry| {
        let directory = entry.file_type().is_some_and(|kind| kind.is_dir());
        if directory && (entry.file_name() == ".git" || entry.file_name() == ".jj") {
            return false;
        }
        let regular = entry.file_type().is_some_and(|kind| kind.is_file());
        if let Some(error) = envelope::entry_refusal(
            entry.path(),
            entry.path_is_symlink(),
            directory,
            regular,
            &repository,
            &root_json,
            &custom,
        ) {
            if let Ok(mut slot) = observed.lock()
                && slot.is_none()
            {
                *slot = Some(error);
            }
            return false;
        }
        true
    });
    for entry in builder.build() {
        let entry = entry.map_err(|error| Refusal::io(&selection.target, error))?;
        if let Some(error) = entry.error() {
            return Err(Refusal::new(
                RefusalKind::IgnoreSyntax,
                entry.path(),
                error.to_compact_string(),
            ));
        }
        if entry.file_type().is_some_and(|kind| kind.is_dir()) {
            envelope::record_directory(request, entry.path(), selection)?;
            continue;
        }
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        if [".min.", "-min.", "_min."]
            .iter()
            .any(|part| name.contains(part))
            || !path
                .extension()
                .is_some_and(|extension| extension == "html" || extension == "htm")
        {
            continue;
        }
        if path.starts_with(request.cwd)
            && config.matched_path_or_any_parents(path, false).is_ignore()
        {
            continue;
        }
        let relative = path.strip_prefix(request.cwd).map_err(|error| {
            Refusal::new(RefusalKind::OutsideCwd, path, error.to_compact_string())
        })?;
        selection.originals.push(Original {
            path: path.to_path_buf(),
            cwd_relative: relative.to_path_buf(),
            origin: if is_dir {
                Origin::DirectoryDiscovery
            } else {
                Origin::ExplicitFile
            },
            bytes: fs::read(path).map_err(|error| Refusal::io(path, error))?,
        });
    }
    if let Some(error) = refusal
        .lock()
        .map_err(|error| {
            Refusal::new(
                RefusalKind::Internal,
                &selection.target,
                error.to_compact_string(),
            )
        })?
        .take()
    {
        return Err(error);
    }
    Ok(())
}
