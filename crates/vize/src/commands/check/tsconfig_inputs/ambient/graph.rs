//! Configured declaration reachability, retaining installed reference ownership.

use std::{
    fs,
    path::{Path, PathBuf},
};
use vize_l0::{FxHashMap, FxHashSet};

use super::{
    DeclarationContents, NODE_MODULES_DIR, ReferenceRealpaths, is_declaration_file,
    path_has_component, reference_path_declaration_files, reference_type_declaration_files,
};

/// Configured declaration membership before ambient-only filtering. Nuxt's
/// generated stubs must not invent globals from unrelated hidden siblings.
pub(crate) fn collect_declaration_reference_graph(
    configured_files: &[PathBuf],
) -> FxHashSet<PathBuf> {
    collect_declaration_reference_graph_reading(
        configured_files,
        DeclarationContents::default(),
        |_, _| {},
    )
}

/// [`collect_declaration_reference_graph`], handing `read` the path and content
/// of every declaration in the graph as it is read, so a caller that inspects
/// them reads each file once. A declaration whose text is in `contents`
/// (a package graph walk already read it) is taken from there, not read again.
pub(crate) fn collect_declaration_reference_graph_reading(
    configured_files: &[PathBuf],
    contents: DeclarationContents,
    read: impl FnMut(&Path, &str),
) -> FxHashSet<PathBuf> {
    reference_graph_reading(configured_files, contents, false, read).files
}

/// [`collect_declaration_reference_graph_reading`], following `path`
/// references into installed packages as well, as `tsc` does. Nuxt's
/// generated `nuxt.d.ts` reaches Nitro's augmentations only that way.
pub(crate) struct InstalledDeclarationGraph {
    pub(crate) files: FxHashSet<PathBuf>,
    pub(crate) installed: FxHashSet<PathBuf>,
    pub(crate) references: FxHashMap<PathBuf, Vec<PathBuf>>,
    pub(crate) directives: FxHashMap<PathBuf, Vec<DeclarationReference>>,
}

/// Directive targets resolved during the existing walk, including path targets
/// outside the declaration-only graph. Snapshotting never resolves them again.
pub(crate) enum DeclarationReference {
    Path {
        specifier: vize_l0::String,
        target: PathBuf,
    },
    Types {
        specifier: vize_l0::String,
        targets: Vec<PathBuf>,
    },
}

impl DeclarationReference {
    pub(crate) fn key(&self) -> (&'static str, &str) {
        match self {
            Self::Path { specifier, .. } => ("path", specifier.as_str()),
            Self::Types { specifier, .. } => ("types", specifier.as_str()),
        }
    }
}

pub(crate) fn collect_installed_declaration_reference_graph_reading(
    configured_files: &[PathBuf],
    contents: DeclarationContents,
    read: impl FnMut(&Path, &str),
) -> InstalledDeclarationGraph {
    reference_graph_reading(configured_files, contents, true, read)
}

fn reference_graph_reading(
    configured_files: &[PathBuf],
    mut contents: DeclarationContents,
    installed: bool,
    mut read: impl FnMut(&Path, &str),
) -> InstalledDeclarationGraph {
    // Reuse the program's already collected roots. Do not walk the workspace
    // or parse its configuration again for generated-stub discovery.
    let mut files = configured_files.to_vec();
    // Configured roots may live outside the app (Nuxt's custom buildDir).
    // Their explicit references retain membership across that boundary.
    let mut graph = follow_declaration_references(installed, &mut files, &mut contents, &mut read);
    let mut pending = graph.installed.iter().cloned().collect::<Vec<_>>();
    while let Some(parent) = pending.pop() {
        for target in graph.references.get(&parent).into_iter().flatten() {
            if graph.installed.insert(target.clone()) {
                pending.push(target.clone());
            }
        }
    }
    graph.files = files
        .into_iter()
        .filter(|path| is_declaration_file(path))
        .collect();
    graph
}

fn follow_declaration_references(
    installed: bool,
    files: &mut Vec<PathBuf>,
    contents: &mut DeclarationContents,
    read: &mut dyn FnMut(&Path, &str),
) -> InstalledDeclarationGraph {
    let mut seen = files.iter().cloned().collect::<FxHashSet<_>>();
    let mut realpaths = ReferenceRealpaths::default();
    let mut edges = FxHashMap::default();
    let mut directives = FxHashMap::default();
    if installed {
        realpaths.installed.extend(
            files
                .iter()
                .filter(|path| path_has_component(path, NODE_MODULES_DIR))
                .cloned(),
        );
    }
    let mut index = 0;
    while let Some(path) = files.get(index).cloned() {
        index += 1;
        if !is_declaration_file(&path) {
            continue;
        }
        let Some(content) = contents
            .remove(&path)
            .or_else(|| fs::read_to_string(&path).ok())
        else {
            continue;
        };
        read(&path, &content);
        let mut resolved = Vec::new();
        let mut referenced = reference_path_declaration_files(
            &path,
            &content,
            None,
            installed,
            &mut realpaths,
            |specifier, target| {
                if installed {
                    resolved.push(DeclarationReference::Path {
                        specifier: specifier.into(),
                        target: target.to_path_buf(),
                    });
                }
            },
        );
        let type_references =
            reference_type_declaration_files(&path, &content, contents, |specifier, targets| {
                if installed {
                    resolved.push(DeclarationReference::Types {
                        specifier: specifier.into(),
                        targets: targets.to_vec(),
                    });
                }
            });
        if installed {
            realpaths.installed.extend(type_references.iter().cloned());
        }
        referenced.extend(type_references);
        if installed && realpaths.installed.contains(&path) {
            realpaths.installed.extend(referenced.iter().cloned());
        }
        for target in &referenced {
            if seen.insert(target.clone()) {
                files.push(target.clone());
            }
        }
        if installed {
            if !resolved.is_empty() {
                directives.insert(path.clone(), resolved);
            }
            edges.insert(path, referenced);
        }
    }
    InstalledDeclarationGraph {
        files: FxHashSet::default(),
        installed: realpaths.installed,
        references: edges,
        directives,
    }
}
