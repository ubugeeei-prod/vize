//! The project-reference graph of one solution, read through a host.

use std::path::{Path, PathBuf};

use serde_json::Value;

use super::{
    OwnershipSpec, TsconfigSourceKind, extends_entries, host_uses_case_sensitive_file_names,
    normalize_path, reference_config_candidates, reference_entries, source_spellings, unique_owner,
};
use crate::typescript_project::CompilerConfigHost;
use crate::{FxHashMap, FxHashSet};

/// One solution graph read through a host's observed configuration reads:
/// the root followed by every transitive reference, with each project's spec.
#[derive(Clone, Debug)]
pub struct ProjectReferenceGraph {
    projects: Vec<PathBuf>,
    specs: FxHashMap<PathBuf, OwnershipSpec>,
    case_sensitive: bool,
}

impl ProjectReferenceGraph {
    /// Read the graph rooted at `root`. Missing references are dropped and an
    /// unreadable config owns nothing, as TypeScript reports and skips them.
    pub fn load<H: CompilerConfigHost + ?Sized>(root: &Path, host: &H) -> Self {
        let mut loader = SpecLoader {
            host,
            values: FxHashMap::default(),
            active: FxHashSet::default(),
            completed: FxHashMap::default(),
        };
        let mut projects = Vec::new();
        let mut seen = FxHashSet::default();
        let mut pending = vec![normalize_path(root)];
        while let Some(project) = pending.pop() {
            if !seen.insert(project.clone()) {
                continue;
            }
            projects.push(project.clone());
            let Some(config) = loader.read(&project) else {
                continue;
            };
            // Depth-first in declaration order: push in reverse.
            let mut references = reference_entries(&config)
                .into_iter()
                .filter_map(|reference| {
                    reference_config_candidates(&project, reference)
                        .into_iter()
                        .find(|candidate| {
                            host.observe(candidate);
                            candidate.is_file()
                        })
                        .map(|candidate| normalize_path(&candidate))
                })
                .collect::<Vec<_>>();
            references.reverse();
            pending.extend(references);
        }
        let specs = projects
            .iter()
            .filter_map(|project| Some((project.clone(), loader.load(project)?)))
            .collect();
        Self {
            projects,
            specs,
            case_sensitive: host_uses_case_sensitive_file_names(),
        }
    }

    /// Whether the root references any project at all.
    pub fn has_references(&self) -> bool {
        self.projects.len() > 1
    }

    /// The root followed by every transitive reference.
    pub fn projects(&self) -> &[PathBuf] {
        &self.projects
    }

    /// The project owning `source`, spelled as the host spells it; see
    /// [`source_spellings`].
    pub fn select(&self, source: &Path, kind: TsconfigSourceKind) -> Option<PathBuf> {
        self.select_spellings(&source_spellings(source), kind)
    }

    /// [`Self::select`] for spellings the host already derived.
    pub fn select_spellings(
        &self,
        spellings: &[PathBuf],
        kind: TsconfigSourceKind,
    ) -> Option<PathBuf> {
        let root = self.projects.first()?;
        Some(unique_owner(root, &self.projects, |project| {
            self.specs
                .get(project)
                .is_some_and(|spec| spec.includes_any(spellings, self.case_sensitive, kind))
        }))
    }
}

struct SpecLoader<'a, H: CompilerConfigHost + ?Sized> {
    host: &'a H,
    /// Each config is read once, for both the graph walk and its spec.
    values: FxHashMap<PathBuf, Option<Value>>,
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, Option<OwnershipSpec>>,
}

impl<H: CompilerConfigHost + ?Sized> SpecLoader<'_, H> {
    fn read(&mut self, config: &Path) -> Option<Value> {
        self.values
            .entry(config.to_path_buf())
            .or_insert_with(|| self.host.read_jsonc_observed(config).ok())
            .clone()
    }

    fn load(&mut self, config: &Path) -> Option<OwnershipSpec> {
        let config = normalize_path(config);
        if let Some(cached) = self.completed.get(&config) {
            return cached.clone();
        }
        if !self.active.insert(config.clone()) {
            return None;
        }
        let loaded = self.read(&config).map(|value| {
            let parents = extends_entries(&value)
                .into_iter()
                .filter_map(|parent| self.host.resolve_extends(&config, parent))
                .filter_map(|parent| self.load(&parent))
                .collect::<Vec<_>>();
            OwnershipSpec::from_config(&config, &value, parents)
        });
        self.active.remove(&config);
        self.completed.insert(config, loaded.clone());
        loaded
    }
}
