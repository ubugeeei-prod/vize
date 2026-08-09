//! Manual import-path rewriting for file renames.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

mod path;
mod script;

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use ignore::{WalkBuilder, WalkState};
use tower_lsp::lsp_types::{FileRename, Range, TextEdit, Url, WorkspaceEdit};

pub(super) use self::path::{
    RESOLVABLE_SCRIPT_EXTENSIONS, apply_all_path_renames, candidate_exists, normalize_path_buf,
    relative_module_path, rewrite_relative_specifier, split_specifier_suffix, strip_extension,
};
use self::script::{collect_script_file_edits, collect_vue_edits};
use crate::{ide::offset_to_position, server::ServerState};

const SCRIPT_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"];

pub(super) struct RenameTarget {
    old_path: PathBuf,
    new_path: PathBuf,
}

#[derive(Clone, Copy)]
enum ImporterKind {
    Vue,
    Script,
}

struct OpenDocumentSnapshot {
    uri: Url,
    source: std::string::String,
}

type OpenDocumentSnapshots = HashMap<PathBuf, OpenDocumentSnapshot>;

pub(super) fn collect_import_rename_edits(
    state: &ServerState,
    renames: &[FileRename],
    only_vue_importers: bool,
) -> Option<WorkspaceEdit> {
    let rename_targets = rename_targets(renames);
    if rename_targets.is_empty() {
        return None;
    }

    let workspace_root = workspace_root(state);
    let open_documents = snapshot_open_documents(state);
    let changes = Mutex::new(HashMap::new());
    let seen_paths = Mutex::new(HashSet::new());

    WalkBuilder::new(&workspace_root)
        .standard_filters(true)
        .hidden(true)
        .build_parallel()
        .run(|| {
            let changes = &changes;
            let seen_paths = &seen_paths;
            let open_documents = &open_documents;
            let rename_targets = &rename_targets;

            Box::new(move |entry| {
                let Ok(entry) = entry else {
                    return WalkState::Continue;
                };

                let path = entry.path();
                let Some(kind) = importer_kind(path, only_vue_importers) else {
                    return WalkState::Continue;
                };

                if let Ok(mut seen) = seen_paths.lock() {
                    seen.insert(normalize_path_buf(path));
                }

                if let Some((uri, edits)) =
                    process_importer_path(state, path, kind, rename_targets, open_documents)
                    && let Ok(mut changes) = changes.lock()
                {
                    changes.insert(uri, edits);
                }

                WalkState::Continue
            })
        });

    let seen_paths = seen_paths.into_inner().unwrap_or_default();
    for (path, document) in open_documents {
        let Some(kind) = importer_kind(&path, only_vue_importers) else {
            continue;
        };
        if seen_paths.contains(&path) {
            continue;
        }

        if let Some((uri, edits)) = process_source(
            state,
            document.uri,
            &path,
            &document.source,
            kind,
            &rename_targets,
        ) && let Ok(mut changes) = changes.lock()
        {
            changes.insert(uri, edits);
        }
    }

    let changes = changes.into_inner().unwrap_or_default();
    if changes.is_empty() {
        None
    } else {
        Some(WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        })
    }
}

pub(super) fn rename_open_documents(
    state: &ServerState,
    renames: &[FileRename],
) -> Vec<(Url, Url)> {
    let rename_targets = rename_targets(renames);
    if rename_targets.is_empty() {
        return Vec::new();
    }

    let mut renamed_documents = Vec::new();
    let open_uris = state.documents.uris();

    for old_uri in open_uris {
        let Some(new_uri) = apply_all_uri_renames(&old_uri, &rename_targets) else {
            continue;
        };

        if new_uri == old_uri {
            continue;
        }

        if state.rename_document(&old_uri, new_uri.clone()) {
            state.remove_virtual_docs(&old_uri);

            if let Some(document) = state.documents.get(&new_uri) {
                let content = document.text();
                drop(document);
                state.update_virtual_docs(&new_uri, &content);
            }

            renamed_documents.push((old_uri, new_uri));
        }
    }

    renamed_documents
}

fn process_importer_path(
    state: &ServerState,
    path: &Path,
    kind: ImporterKind,
    rename_targets: &[RenameTarget],
    open_documents: &OpenDocumentSnapshots,
) -> Option<(Url, Vec<TextEdit>)> {
    let normalized_path = normalize_path_buf(path);
    if let Some(document) = open_documents.get(&normalized_path) {
        return process_source(
            state,
            document.uri.clone(),
            &normalized_path,
            &document.source,
            kind,
            rename_targets,
        );
    }

    let uri = Url::from_file_path(&normalized_path).ok()?;
    let source = fs::read_to_string(&normalized_path).ok()?;
    process_source(state, uri, &normalized_path, &source, kind, rename_targets)
}

fn process_source(
    state: &ServerState,
    uri: Url,
    path: &Path,
    source: &str,
    kind: ImporterKind,
    rename_targets: &[RenameTarget],
) -> Option<(Url, Vec<TextEdit>)> {
    let future_path =
        apply_all_path_renames(path, rename_targets).unwrap_or_else(|| normalize_path_buf(path));

    let mut edits = match kind {
        ImporterKind::Vue => collect_vue_edits(state, path, &future_path, source, rename_targets),
        ImporterKind::Script => {
            collect_script_file_edits(state, path, &future_path, source, rename_targets)
        }
    };

    if edits.is_empty() {
        return None;
    }

    edits.sort_by(|left, right| {
        left.range
            .start
            .line
            .cmp(&right.range.start.line)
            .then(left.range.start.character.cmp(&right.range.start.character))
    });

    Some((uri, edits))
}

fn rename_targets(renames: &[FileRename]) -> Vec<RenameTarget> {
    renames
        .iter()
        .filter_map(|rename| {
            let old_path = Url::parse(&rename.old_uri).ok()?.to_file_path().ok()?;
            let new_path = Url::parse(&rename.new_uri).ok()?.to_file_path().ok()?;

            Some(RenameTarget {
                old_path: normalize_path_buf(&old_path),
                new_path: normalize_path_buf(&new_path),
            })
        })
        .collect()
}

fn apply_all_uri_renames(uri: &Url, renames: &[RenameTarget]) -> Option<Url> {
    let path = uri.to_file_path().ok()?;
    let path = apply_all_path_renames(&path, renames)?;
    Url::from_file_path(path).ok()
}

fn importer_kind(path: &Path, only_vue_importers: bool) -> Option<ImporterKind> {
    if path.extension().is_some_and(|extension| extension == "vue") {
        return Some(ImporterKind::Vue);
    }

    if only_vue_importers {
        return None;
    }

    let extension = path.extension()?.to_str()?;
    if SCRIPT_EXTENSIONS.contains(&extension) {
        Some(ImporterKind::Script)
    } else {
        None
    }
}

fn snapshot_open_documents(state: &ServerState) -> OpenDocumentSnapshots {
    let mut snapshots = HashMap::with_capacity(state.documents.len());
    for document in state.documents.iter() {
        let Ok(path) = document.key().to_file_path() else {
            continue;
        };
        snapshots.insert(
            normalize_path_buf(&path),
            OpenDocumentSnapshot {
                uri: document.key().clone(),
                source: document.value().text(),
            },
        );
    }
    snapshots
}

pub(super) fn offset_range(source: &str, start: usize, end: usize) -> Option<Range> {
    let (start_line, start_character) = offset_to_position(source, start);
    let (end_line, end_character) = offset_to_position(source, end);

    Some(Range {
        start: tower_lsp::lsp_types::Position {
            line: start_line,
            character: start_character,
        },
        end: tower_lsp::lsp_types::Position {
            line: end_line,
            character: end_character,
        },
    })
}

#[cfg(feature = "native")]
fn workspace_root(state: &ServerState) -> PathBuf {
    state
        .get_workspace_root()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[cfg(not(feature = "native"))]
fn workspace_root(_state: &ServerState) -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
